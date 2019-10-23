use crate::gene::Gene;
use crate::lookup;
use crate::stack;
use crate::stack::{nr_to_bool, Stack};
use crate::triplet::{Mode, Triplet};

pub struct Processor<'a> {
    gene: &'a Gene<'a>,
    stack: Vec<u32>,
    pc: usize,
    failures: u32,
}

pub struct ExecutionContext<'a> {
    max_stack_size: usize,
    instruction_lookup: &'a lookup::Lookup<Instruction>,
}

impl<'a> Processor<'a> {
    pub fn new(gene: &'a Gene<'a>) -> Processor {
        return Processor {
            gene,
            stack: vec![],
            pc: 0,
            failures: 0,
        };
    }

    pub fn execute(&mut self, context: &ExecutionContext) {
        let code = self.gene.code;

        let value = code[self.pc];
        self.pc += 1;
        if self.pc >= code.len() {
            self.pc = 0;
        }
        let t = Triplet::from_int(value);
        match t.mode {
            Mode::Number => {
                self.stack.push(value);
            }
            Mode::Instruction => {
                let instruction = context.instruction_lookup.find(value);
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
        if new_pc < 0 || new_pc >= (self.gene.code.len() as i32) {
            return None;
        }
        self.pc = new_pc as usize;
        Some(())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ProcessorInstruction {
    JF = 0x010100,
    JB = 0x010110,
}

impl<'a> ProcessorInstruction {
    pub fn execute(&self, processor: &mut Processor<'a>) -> Option<()> {
        match self {
            ProcessorInstruction::JF => processor.stack.pop2().and_then(|(first, second)| {
                if !nr_to_bool(first) {
                    return Some(());
                }
                if second == 0 {
                    return Some(());
                }
                processor.jump(second as i32)
            }),
            ProcessorInstruction::JB => processor.stack.pop2().and_then(|(first, second)| {
                if !nr_to_bool(first) {
                    return Some(());
                }
                if second == 0 {
                    return Some(());
                }
                processor.jump(-(second as i32 + 1))
            }),
        }
    }

    pub fn coordinates(&self) -> u32 {
        *self as u32
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    StackInstruction(stack::Instruction),
    ProcessorInstruction(ProcessorInstruction),
}

impl<'a> Instruction {
    pub fn execute(&self, processor: &mut Processor<'a>) -> Option<()> {
        match self {
            Instruction::StackInstruction(instruction) => instruction.execute(&mut processor.stack),
            Instruction::ProcessorInstruction(instruction) => instruction.execute(processor),
        }
    }
}

impl lookup::Coordinates for Instruction {
    fn coordinates(&self) -> u32 {
        match self {
            Instruction::StackInstruction(instruction) => instruction.coordinates(),
            Instruction::ProcessorInstruction(instruction) => instruction.coordinates(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::stack;
    const ADD_NR: u32 = stack::Instruction::Add as u32 | 0x01000000;
    const SUB_NR: u32 = stack::Instruction::Sub as u32 | 0x01000000;
    const DUP_NR: u32 = stack::Instruction::Dup as u32 | 0x01000000;
    const JF_NR: u32 = ProcessorInstruction::JF as u32 | 0x01000000;
    const JB_NR: u32 = ProcessorInstruction::JB as u32 | 0x01000000;
    fn instruction_lookup<'a>() -> lookup::Lookup<Instruction> {
        let mut l = lookup::Lookup::<Instruction>::new();

        l.add(Instruction::StackInstruction(stack::Instruction::Add))
            .expect("cannot add");
        l.add(Instruction::StackInstruction(stack::Instruction::Sub))
            .expect("cannot add");
        l.add(Instruction::StackInstruction(stack::Instruction::Dup))
            .expect("cannot add");
        l.add(Instruction::ProcessorInstruction(ProcessorInstruction::JF))
            .expect("cannot add");
        l.add(Instruction::ProcessorInstruction(ProcessorInstruction::JB))
            .expect("cannot add");

        return l;
    }
    #[test]
    fn test_processor_execute() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let gene = Gene::new(&[3, 4, ADD_NR]);
        let mut g = Processor::new(&gene);

        g.execute_amount(&context, 3);

        assert_eq!(g.stack, [7]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_processor_execute_multiple() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let gene = Gene::new(&[3, 4, ADD_NR, 6, SUB_NR]);
        let mut g = Processor::new(&gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [1]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_processor_execute_beyond_end() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let gene = Gene::new(&[3, 4, ADD_NR]);
        let mut g = Processor::new(&gene);

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
    fn test_processor_execute_nearby() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let gene = Gene::new(&[3, 4, ADD_NR + 1, 6, SUB_NR - 1]);
        let mut g = Processor::new(&gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [1]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_processor_execute_stack_underflow() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let gene = Gene::new(&[4, ADD_NR]);
        let mut g = Processor::new(&gene);

        g.execute_amount(&context, 2);

        assert_eq!(g.stack, []);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_processor_execute_stack_overflow_numbers() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 4,
        };

        let gene = Gene::new(&[1, 2, 3, 4, 5]);
        let mut g = Processor::new(&gene);

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
    fn test_processor_execute_stack_overflow_instructions() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 4,
        };

        let gene = Gene::new(&[1, DUP_NR, DUP_NR, DUP_NR, DUP_NR]);
        let mut g = Processor::new(&gene);

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

        let gene = Gene::new(&[1, 1, JF_NR, 66, 77]);
        let mut g = Processor::new(&gene);

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

        let gene = Gene::new(&[1, 2, JF_NR, 66, 77, 88]);
        let mut g = Processor::new(&gene);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [88]);
    }

    #[test]
    fn test_jf_too_far() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let gene = Gene::new(&[1, 200, JF_NR, 66, 88]);
        let mut g = Processor::new(&gene);

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

        let gene = Gene::new(&[0, 1, JF_NR, 66, 88]);
        let mut g = Processor::new(&gene);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [66]);
    }

    #[test]
    fn test_jf_zero() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let gene = Gene::new(&[1, 0, JF_NR, 66, 88]);
        let mut g = Processor::new(&gene);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [66]);
    }

    #[test]
    fn test_jb() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let gene = Gene::new(&[88, 1, 3, JB_NR, 66]);
        let mut g = Processor::new(&gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 88]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_jb_false() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let gene = Gene::new(&[88, 0, 3, JB_NR, 66]);
        let mut g = Processor::new(&gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 66]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_jb_1() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let gene = Gene::new(&[88, 1, 1, JB_NR, 66]);
        let mut g = Processor::new(&gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 1]);
    }

    #[test]
    fn test_jb_zero() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let gene = Gene::new(&[88, 1, 0, JB_NR, 66]);
        let mut g = Processor::new(&gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 66]);
    }

    #[test]
    fn test_jb_too_far() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let gene = Gene::new(&[88, 1, 100, JB_NR, 66]);

        let mut g = Processor::new(&gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 66]);
        assert_eq!(g.failures, 1);
    }

}
