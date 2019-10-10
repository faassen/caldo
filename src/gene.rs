use crate::instruction_lookup::InstructionLookup;
use crate::stack;
use crate::stack::{nr_to_bool, Stack};
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
                let success = instruction.execute(self);
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

    pub fn execute_amount(&mut self, context: &ExecutionContext, amount: usize) {
        (0..amount).for_each(|_| self.execute(context))
    }

    pub fn shrink_stack_on_overflow(&mut self, context: &ExecutionContext) {
        if self.stack.len() <= context.max_stack_size {
            return;
        }
        self.failures += 1;
        self.stack
            .splice(..context.max_stack_size / 2, [].iter().cloned());
    }

    fn jump(&mut self, adjust: i32) -> Option<()> {
        let new_pc: i32 = (self.pc as i32) + adjust;
        if new_pc < 0 || new_pc >= (self.code.len() as i32) {
            return None;
        }
        self.pc = new_pc as usize;
        Some(())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GeneInstruction {
    JF,
    JB,
}

impl<'a> GeneInstruction {
    pub fn execute(&self, gene: &mut Gene<'a>) -> Option<()> {
        match self {
            GeneInstruction::JF => gene.stack.pop2().and_then(|(first, second)| {
                if !nr_to_bool(first) {
                    return Some(());
                }
                if second == 0 {
                    return Some(());
                }
                gene.jump(second as i32)
            }),
            GeneInstruction::JB => gene.stack.pop2().and_then(|(first, second)| {
                if !nr_to_bool(first) {
                    return Some(());
                }
                if second == 0 {
                    return Some(());
                }
                gene.jump(-(second as i32 + 1))
            }),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    StackInstruction(stack::Instruction),
    GeneInstruction(GeneInstruction),
}

impl<'a> Instruction {
    pub fn execute(&self, gene: &mut Gene<'a>) -> Option<()> {
        match self {
            Instruction::StackInstruction(instruction) => instruction.execute(&mut gene.stack),
            Instruction::GeneInstruction(instruction) => instruction.execute(gene),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::stack;

    const ADD_NR: u32 = 0x01010203;
    const SUB_NR: u32 = 0x01030201;
    const DUP_NR: u32 = 0x01070707;
    const JF_NR: u32 = 0x010F0F0F;
    const JB_NR: u32 = 0x010E0F0F;
    fn instruction_lookup() -> InstructionLookup {
        let mut l = InstructionLookup::new();
        let add_triplet = Triplet::from_int(ADD_NR);
        let sub_triplet = Triplet::from_int(SUB_NR);
        let dup_triplet = Triplet::from_int(DUP_NR);
        let jf_triplet = Triplet::from_int(JF_NR);
        let jb_triplet = Triplet::from_int(JB_NR);

        l.add(
            add_triplet,
            Instruction::StackInstruction(stack::Instruction::Add),
        )
        .expect("cannot add");
        l.add(
            sub_triplet,
            Instruction::StackInstruction(stack::Instruction::Sub),
        )
        .expect("cannot add");
        l.add(
            dup_triplet,
            Instruction::StackInstruction(stack::Instruction::Dup),
        )
        .expect("cannot add");
        l.add(
            jf_triplet,
            Instruction::GeneInstruction(GeneInstruction::JF),
        )
        .expect("cannot add");
        l.add(
            jb_triplet,
            Instruction::GeneInstruction(GeneInstruction::JB),
        )
        .expect("cannot add");

        return l;
    }
    #[test]
    fn test_gene_execute() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[3, 4, ADD_NR]);

        g.execute_amount(&context, 3);

        assert_eq!(g.stack, [7]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_gene_execute_multiple() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[3, 4, ADD_NR, 6, SUB_NR]);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [1]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_gene_execute_beyond_end() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[3, 4, ADD_NR]);

        g.execute_amount(&context, 6);

        // 3
        // 4
        // 7
        // 7 3
        // 7 3 4
        // 7 7

        assert_eq!(g.stack, [7, 7]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_gene_execute_nearby() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[3, 4, ADD_NR + 1, 6, SUB_NR - 1]);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [1]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_gene_execute_stack_underflow() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[4, ADD_NR]);

        g.execute_amount(&context, 2);

        assert_eq!(g.stack, []);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_gene_execute_stack_overflow_numbers() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 4,
        };

        let mut g = Gene::new(&[1, 2, 3, 4, 5]);

        g.execute_amount(&context, 5);

        // 1
        // 1 2
        // 1 2 3
        // 1 2 3 4
        // 3 4 5

        assert_eq!(g.stack, [3, 4, 5]);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_gene_execute_stack_overflow_instructions() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 4,
        };

        let mut g = Gene::new(&[1, DUP_NR, DUP_NR, DUP_NR, DUP_NR]);

        g.execute_amount(&context, 5);

        // 1
        // 1 1
        // 1 1 1
        // 1 1 1 1
        // 1 1 1 1 1
        assert_eq!(g.stack, [1, 1, 1]);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_jf() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[1, 1, JF_NR, 66, 77]);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [77]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_jf2() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[1, 2, JF_NR, 66, 77, 88]);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [88]);
    }

    #[test]
    fn test_jf_too_far() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[1, 200, JF_NR, 66, 88]);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [66]);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_jf_false() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[0, 1, JF_NR, 66, 88]);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [66]);
    }

    #[test]
    fn test_jf_zero() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[1, 0, JF_NR, 66, 88]);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [66]);
    }

    #[test]
    fn test_jb() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[88, 1, 3, JB_NR, 66]);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 88]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_jb_1() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[88, 1, 1, JB_NR, 66]);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 1]);
    }

    #[test]
    fn test_jb_zero() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[88, 1, 0, JB_NR, 66]);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 66]);
    }

    #[test]
    fn test_jb_too_far() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[88, 1, 100, JB_NR, 66]);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 66]);
        assert_eq!(g.failures, 1);
    }

}
