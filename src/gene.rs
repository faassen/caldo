use crate::stack::Instruction;
use crate::triplet::{Mode, Triplet};
use kdtree::distance::squared_euclidean;
use kdtree::ErrorKind;
use kdtree::KdTree;

pub struct Gene<'a> {
    code: &'a [u32],
    stack: Vec<u32>,
    pc: usize,
    failures: u32,
}

pub struct ExecutionContext<'a> {
    max_stack_size: usize,
    instruction_lookup: &'a InstructionLookup,
}

impl<'a> Gene<'a> {
    pub fn new(code: &[u32]) -> Gene {
        return Gene {
            code: code,
            stack: vec![],
            pc: 0,
            failures: 0,
        };
    }

    pub fn execute(&mut self, context: &ExecutionContext) {
        let value = self.code[self.pc];
        self.pc += 1;
        if self.pc >= self.code.len() {
            self.pc = 0;
        }
        let t = Triplet::from_int(value);
        match t.mode {
            Mode::Number => {
                self.stack.push(value);
            }
            Mode::Instruction => {
                let instruction = context.instruction_lookup.find(t);
                let success = instruction.execute(&mut self.stack);
                match success {
                    None => {
                        self.failures += 1;
                    }
                    Some(_) => {}
                }
            }
            Mode::Call => {}
            Mode::Noop => {
                return;
            }
        }
        self.shrink_stack_on_overflow(context);
    }

    pub fn shrink_stack_on_overflow(&mut self, context: &ExecutionContext) {
        if self.stack.len() <= context.max_stack_size {
            return;
        }
        self.failures += 1;
        self.stack
            .splice(..context.max_stack_size / 2, [].iter().cloned());
    }
}

pub struct InstructionLookup {
    tree: KdTree<f32, Instruction, [f32; 3]>,
}

#[derive(Debug, Clone, Copy)]
pub struct InstructionLookupError {}

pub type InstructionLookupAddResult = Result<(), InstructionLookupError>;

impl From<ErrorKind> for InstructionLookupError {
    fn from(_error: ErrorKind) -> Self {
        InstructionLookupError {}
    }
}

impl InstructionLookup {
    pub fn new() -> InstructionLookup {
        return InstructionLookup {
            tree: KdTree::new(3),
        };
    }

    pub fn add(
        &mut self,
        triplet: Triplet,
        instruction: Instruction,
    ) -> InstructionLookupAddResult {
        self.tree.add(triplet.coordinates(), instruction)?;
        return Ok(());
    }

    pub fn find(&self, t: Triplet) -> Instruction {
        let v = self
            .tree
            .nearest(&t.coordinates(), 1, &squared_euclidean)
            .unwrap();
        return *v[0].1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_lookup_identify() -> InstructionLookupAddResult {
        let mut l = InstructionLookup::new();
        let t = Triplet::from_int(0x010203);
        let i = Instruction::Add;
        l.add(t, i)?;
        assert_eq!(l.find(t), i);
        return Ok(());
    }

    #[test]
    fn test_instruction_lookup_near() -> InstructionLookupAddResult {
        let mut l = InstructionLookup::new();
        let t1 = Triplet::from_int(0x010203);
        let t2 = Triplet::from_int(0xFFFFFF);
        let tlookup = Triplet::from_int(0x010402);

        let i1 = Instruction::Add;
        let i2 = Instruction::Sub;
        l.add(t1, i1)?;
        l.add(t2, i2)?;
        assert_eq!(l.find(tlookup), i1);
        return Ok(());
    }

    const ADD_NR: u32 = 0x01010203;
    const SUB_NR: u32 = 0x01030201;
    const DUP_NR: u32 = 0x01070707;

    fn instruction_lookup() -> InstructionLookup {
        let mut l = InstructionLookup::new();
        let add_triplet = Triplet::from_int(ADD_NR);
        let sub_triplet = Triplet::from_int(SUB_NR);
        let dup_triplet = Triplet::from_int(DUP_NR);
        l.add(add_triplet, Instruction::Add).expect("cannot add");
        l.add(sub_triplet, Instruction::Sub).expect("cannot add");
        l.add(dup_triplet, Instruction::Dup).expect("cannot add");
        return l;
    }
    #[test]
    fn test_gene_execute() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[3, 4, ADD_NR]);

        g.execute(&context);
        g.execute(&context);
        g.execute(&context);

        assert_eq!(g.stack.len(), 1);
        assert_eq!(g.stack[0], 7);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_gene_execute_multiple() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[3, 4, ADD_NR, 6, SUB_NR]);

        g.execute(&context);
        g.execute(&context);
        g.execute(&context);
        g.execute(&context);
        g.execute(&context);
        assert_eq!(g.stack.len(), 1);
        assert_eq!(g.stack[0], 1);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_gene_execute_beyond_end() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[3, 4, ADD_NR]);

        g.execute(&context); // 3
        g.execute(&context); // 4
        g.execute(&context); // 7
        g.execute(&context); // 7 3
        g.execute(&context); // 7 3 4
        g.execute(&context); // 7 7
        assert_eq!(g.stack.len(), 2);
        assert_eq!(g.stack[0], 7);
        assert_eq!(g.stack[0], 7);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_gene_execute_nearby() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[3, 4, ADD_NR + 1, 6, SUB_NR - 1]);

        g.execute(&context);
        g.execute(&context);
        g.execute(&context);
        g.execute(&context);
        g.execute(&context);
        assert_eq!(g.stack.len(), 1);
        assert_eq!(g.stack[0], 1);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_gene_execute_stack_underflow() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[4, ADD_NR]);

        g.execute(&context);
        g.execute(&context);
        assert_eq!(g.stack.len(), 0);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_gene_execute_stack_overflow_numbers() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 4,
        };

        let mut g = Gene::new(&[1, 2, 3, 4, 5]);

        g.execute(&context); // 1
        g.execute(&context); // 1 2
        g.execute(&context); // 1 2 3
        g.execute(&context); // 1 2 3 4
        g.execute(&context); // 3 4 5
        assert_eq!(g.stack.len(), 3);
        assert_eq!(g.failures, 1);
        assert_eq!(g.stack[0], 3);
        assert_eq!(g.stack[1], 4);
        assert_eq!(g.stack[2], 5);
    }

    #[test]
    fn test_gene_execute_stack_overflow_instructions() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 4,
        };

        let mut g = Gene::new(&[1, DUP_NR, DUP_NR, DUP_NR, DUP_NR]);

        g.execute(&context); // 1
        g.execute(&context); // 1 1
        g.execute(&context); // 1 1 1
        g.execute(&context); // 1 1 1 1
        g.execute(&context); // 1 1 1 1 1
        assert_eq!(g.stack.len(), 3);
        assert_eq!(g.failures, 1);
        assert_eq!(g.stack[0], 1);
        assert_eq!(g.stack[1], 1);
        assert_eq!(g.stack[2], 1);
    }

}
