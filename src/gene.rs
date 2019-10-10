use crate::instruction_lookup::InstructionLookup;
use crate::triplet::{Mode, Triplet};

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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::stack::Instruction;

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
