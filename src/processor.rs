use crate::cell::Cell;
use crate::gene::Gene;
use crate::lookup;
use crate::stack;
use crate::stack::{nr_to_bool, Stack};
use crate::triplet::{Mode, Triplet};

use std::rc::Rc;

pub struct Processor<'a> {
    gene: Rc<Gene<'a>>,
    pub stack: Vec<u32>,
    pub call_stack: Vec<(u32, usize)>,
    pc: usize,
    pub failures: u32,
}

pub struct ExecutionContext<'a> {
    pub max_stack_size: usize,
    pub max_call_stack_size: usize,
    pub instruction_lookup: &'a lookup::Lookup<Instruction>,
    pub cell: &'a Cell<'a>,
}

impl<'a> Processor<'a> {
    pub fn new(gene: Rc<Gene<'a>>) -> Processor<'a> {
        return Processor {
            gene: Rc::clone(&gene),
            stack: vec![],
            call_stack: vec![],
            pc: 0,
            failures: 0,
        };
    }

    pub fn execute(&mut self, context: &'a ExecutionContext) {
        let code = self.gene.code;

        let value = code[self.pc];
        // now increase pc
        self.pc += 1;

        let t = Triplet::from_int(value);
        match t.mode {
            Mode::Number => {
                // println!("number: {}", value);
                self.stack.push(value);
            }
            Mode::Instruction => {
                let instruction = context.instruction_lookup.find(value);
                // println!("value {:x?}, instruction: {:?}", value, instruction);
                let success = instruction.execute(self, context);
                match success {
                    None => {
                        self.failures += 1;
                    }
                    Some(_) => {}
                }
            }
            Mode::Call => {}
            Mode::Noop => {}
        }

        // at the end
        if self.pc >= code.len() {
            let top = self.call_stack.pop();
            match top {
                Some((gene_id, return_pc)) => {
                    // return to calling gene
                    // XXX must check for gene_id being valid
                    self.gene = context.cell.get_gene(gene_id).unwrap();
                    self.pc = return_pc;
                }
                None => {
                    // go back to start
                    self.pc = 0;
                }
            }
        }
        self.shrink_stack_on_overflow(context);
    }

    pub fn execute_amount(&mut self, context: &'a ExecutionContext, amount: usize) {
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

    pub fn shrink_call_stack_on_overflow(&mut self, context: &ExecutionContext) {
        if self.call_stack.len() <= context.max_call_stack_size {
            return;
        }
        self.failures += 1;
        self.call_stack
            .splice(..context.max_call_stack_size / 2, [].iter().cloned());
    }

    fn jump(&mut self, adjust: i32) -> Option<()> {
        let new_pc: i32 = (self.pc as i32) + adjust;
        if new_pc < 0 || new_pc >= (self.gene.code.len() as i32) {
            return None;
        }
        self.pc = new_pc as usize;
        Some(())
    }

    fn call(&mut self, gene_id: u32, context: &'a ExecutionContext) -> Option<()> {
        context.cell.get_gene(gene_id).and_then(|gene| {
            let return_pc = {
                if self.pc >= self.gene.code.len() {
                    0
                } else {
                    self.pc
                }
            };
            self.call_stack.push((self.gene.id, return_pc));
            self.shrink_call_stack_on_overflow(context);
            self.gene = Rc::clone(&gene);
            self.pc = 0;
            Some(())
        })
    }

    fn read_gene(&mut self, gene_id: u32, index: u32, context: &'a ExecutionContext) -> Option<()> {
        context.cell.get_gene(gene_id).and_then(|gene| {
            if index >= gene.code.len() as u32 {
                return None;
            }
            self.stack.push(gene.code[index as usize]);
            Some(())
        })
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ProcessorInstruction {
    JF = 0x010100,
    JB = 0x010110,
    Lookup = 0x010120,
    Call = 0x010130,
    ReadGene = 0x010140,
}

impl<'a> ProcessorInstruction {
    pub fn execute(
        &self,
        processor: &mut Processor<'a>,
        context: &'a ExecutionContext,
    ) -> Option<()> {
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
            ProcessorInstruction::Lookup => processor.stack.pop().and_then(|first| {
                processor.stack.push(context.cell.lookup_gene_id(first));
                Some(())
            }),
            ProcessorInstruction::Call => processor
                .stack
                .pop()
                .and_then(|first| processor.call(first, context)),
            ProcessorInstruction::ReadGene => processor
                .stack
                .pop2()
                .and_then(|(first, second)| processor.read_gene(first, second, context)),
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
    pub fn execute(
        &self,
        processor: &mut Processor<'a>,
        context: &'a ExecutionContext,
    ) -> Option<()> {
        match self {
            Instruction::StackInstruction(instruction) => instruction.execute(&mut processor.stack),
            Instruction::ProcessorInstruction(instruction) => {
                instruction.execute(processor, context)
            }
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
    use rand::SeedableRng;
    const INSTR_BIT: u32 = 0x01000000;
    const ADD_NR: u32 = stack::Instruction::Add as u32 | INSTR_BIT;
    const SUB_NR: u32 = stack::Instruction::Sub as u32 | INSTR_BIT;
    const DUP_NR: u32 = stack::Instruction::Dup as u32 | INSTR_BIT;
    const JF_NR: u32 = ProcessorInstruction::JF as u32 | INSTR_BIT;
    const JB_NR: u32 = ProcessorInstruction::JB as u32 | INSTR_BIT;
    const CALL_NR: u32 = ProcessorInstruction::Call as u32 | INSTR_BIT;
    const LOOKUP_NR: u32 = ProcessorInstruction::Lookup as u32 | INSTR_BIT;
    const READ_GENE_NR: u32 = ProcessorInstruction::ReadGene as u32 | INSTR_BIT;
    const SEED: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

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
        l.add(Instruction::ProcessorInstruction(
            ProcessorInstruction::Lookup,
        ))
        .expect("cannot add");
        l.add(Instruction::ProcessorInstruction(
            ProcessorInstruction::Call,
        ))
        .expect("cannot add");
        l.add(Instruction::ProcessorInstruction(
            ProcessorInstruction::ReadGene,
        ))
        .expect("cannot add");

        return l;
    }
    #[test]
    fn test_processor_execute() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[3, 4, ADD_NR]));
        let mut g = Processor::new(gene);

        g.execute_amount(&context, 3);

        assert_eq!(g.stack, [7]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_processor_execute_multiple() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[3, 4, ADD_NR, 6, SUB_NR]));
        let mut g = Processor::new(gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [1]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_processor_execute_beyond_end() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[3, 4, ADD_NR]));
        let mut g = Processor::new(gene);

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
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[3, 4, ADD_NR + 1, 6, SUB_NR - 1]));
        let mut g = Processor::new(gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [1]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_processor_execute_stack_underflow() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[4, ADD_NR]));
        let mut g = Processor::new(gene);
        g.execute_amount(&context, 2);

        assert_eq!(g.stack, []);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_processor_execute_stack_overflow_numbers() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 4,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[1, 2, 3, 4, 5]));
        let mut g = Processor::new(gene);

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
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 4,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[1, DUP_NR, DUP_NR, DUP_NR, DUP_NR]));
        let mut g = Processor::new(gene);

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
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[1, 1, JF_NR, 66, 77]));
        let mut g = Processor::new(gene);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [77]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_jf2() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[1, 2, JF_NR, 66, 77, 88]));
        let mut g = Processor::new(gene);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [88]);
    }

    #[test]
    fn test_jf_too_far() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[1, 200, JF_NR, 66, 88]));
        let mut g = Processor::new(gene);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [66]);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_jf_false() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[0, 1, JF_NR, 66, 88]));
        let mut g = Processor::new(gene);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [66]);
    }

    #[test]
    fn test_jf_zero() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[1, 0, JF_NR, 66, 88]));
        let mut g = Processor::new(gene);

        g.execute_amount(&context, 4);

        assert_eq!(g.stack, [66]);
    }

    #[test]
    fn test_jb() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[88, 1, 3, JB_NR, 66]));
        let mut g = Processor::new(gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 88]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_jb_false() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[88, 0, 3, JB_NR, 66]));
        let mut g = Processor::new(gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 66]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_jb_1() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[88, 1, 1, JB_NR, 66]));
        let mut g = Processor::new(gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 1]);
    }

    #[test]
    fn test_jb_zero() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[88, 1, 0, JB_NR, 66]));
        let mut g = Processor::new(gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 66]);
    }

    #[test]
    fn test_jb_too_far() {
        let cell = Cell::new();
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };

        let gene = Rc::new(Gene::new(0, &[88, 1, 100, JB_NR, 66]));

        let mut g = Processor::new(gene);

        g.execute_amount(&context, 5);

        assert_eq!(g.stack, [88, 66]);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_lookup() {
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        let gene1_id;
        {
            let gene1 = cell.add_gene(&[3, 4, ADD_NR], &mut rng);
            gene1_id = gene1.id;
        }

        let gene2_id;
        {
            let gene2 = cell.add_gene(&[5, 3, LOOKUP_NR], &mut rng);
            gene2_id = gene2.id;
        }

        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };
        let gene = cell.get_gene(gene2_id).unwrap();
        let mut p = Processor::new(gene);

        p.execute_amount(&context, 3);
        assert_eq!(p.stack, [5, gene1_id]);
    }

    #[test]
    fn test_call_without_return() {
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        cell.add_gene(&[3, 4, ADD_NR], &mut rng);

        let gene2_id;
        {
            // 5 3
            // 5 <NR>
            // 5 3 4
            // 5 7
            let gene2 = cell.add_gene(&[5, 3, LOOKUP_NR, CALL_NR], &mut rng);
            gene2_id = gene2.id;
        }
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };
        let gene = cell.get_gene(gene2_id).unwrap();
        let mut p = Processor::new(gene);

        p.execute_amount(&context, 7);

        assert_eq!(p.stack, [5, 7]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_call_impossible_gene_id() {
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        let gene = cell.add_gene(&[5, CALL_NR, 1, 6, ADD_NR], &mut rng);
        let gene_id = gene.id;
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };
        let gene = cell.get_gene(gene_id).unwrap();
        let mut p = Processor::new(gene);

        p.execute_amount(&context, 5);

        assert_eq!(p.stack, [7]);
        assert_eq!(p.failures, 1);
    }

    #[test]
    fn test_call_and_return() {
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        cell.add_gene(&[3, 4, ADD_NR], &mut rng);

        let gene2_id;
        {
            // 5
            // 5 3
            // 5 <NR>
            // 5
            // 5 3
            // 5 3 4
            // 5 7
            // 5 7 4
            let gene2 = cell.add_gene(&[5, 3, LOOKUP_NR, CALL_NR, 4], &mut rng);
            gene2_id = gene2.id;
        }

        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };
        let gene = cell.get_gene(gene2_id).unwrap();
        let mut p = Processor::new(gene);

        p.execute_amount(&context, 8);

        assert_eq!(p.stack, [5, 7, 4]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_call_at_end() {
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        cell.add_gene(&[3, 4, ADD_NR], &mut rng);

        let gene2_id;
        {
            // 5
            // 5 3
            // 5 <NR>
            // 5
            // 5 3
            // 5 3 4
            // 5 7
            // should wrap again to start
            // 5 7 5
            let gene2 = cell.add_gene(&[5, 3, LOOKUP_NR, CALL_NR], &mut rng);
            gene2_id = gene2.id;
        }

        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };
        let gene = cell.get_gene(gene2_id).unwrap();
        let mut p = Processor::new(gene);

        p.execute_amount(&context, 8);

        assert_eq!(p.stack, [5, 7, 5]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_call_stack_compaction() {
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        cell.add_gene(&[1, 2, LOOKUP_NR, CALL_NR], &mut rng);
        cell.add_gene(&[2, 3, LOOKUP_NR, CALL_NR], &mut rng);
        cell.add_gene(&[3, 4, 10, 20, ADD_NR, 40], &mut rng);
        let gene = cell.add_gene(&[0, 1, LOOKUP_NR, CALL_NR], &mut rng);
        let gene_id = gene.id;
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 2,
            cell: &cell,
        };
        let gene = cell.get_gene(gene_id).unwrap();
        let mut p = Processor::new(gene);

        p.execute_amount(&context, 17);

        assert_eq!(p.stack, [0, 1, 2, 3, 4, 30]);
        assert_eq!(p.call_stack.len(), 2);
        assert_eq!(p.failures, 1);
    }

    #[test]
    fn test_read_gene() {
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        cell.add_gene(&[3, 4, ADD_NR], &mut rng);

        let gene_id;
        {
            let gene = cell.add_gene(&[5, 3, LOOKUP_NR, 0, READ_GENE_NR], &mut rng);
            gene_id = gene.id;
        }

        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
        };
        let gene = cell.get_gene(gene_id).unwrap();
        let mut p = Processor::new(gene);
        p.execute_amount(&context, 5);
        assert_eq!(p.stack, [5, 3]);
    }
}
