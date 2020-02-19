use crate::cell::Cell;
use crate::gene::{Gene, GeneKey};
use crate::lookup;
use crate::stack;
use crate::stack::{nr_to_bool, Stack};
use crate::triplet::{Mode, Triplet};

use slotmap::DenseSlotMap;

pub struct Processor {
    gene_key: GeneKey,
    pub stack: Vec<u32>,
    pub call_stack: Vec<(u32, usize)>,
    pc: usize,
    pub failures: u32,
}

pub struct ExecutionContext<'a> {
    pub max_stack_size: usize,
    pub max_call_stack_size: usize,
    pub instruction_lookup: &'a lookup::Lookup<Instruction>,
    pub cell: &'a Cell,
    pub genes: &'a mut DenseSlotMap<GeneKey, Gene>,
}

impl Processor {
    pub fn new(gene_key: GeneKey) -> Processor {
        return Processor {
            gene_key: gene_key,
            stack: vec![],
            call_stack: vec![],
            pc: 0,
            failures: 0,
        };
    }

    pub fn execute(&mut self, context: &mut ExecutionContext) {
        // XXX is it possible to add gene to execution context?
        let value = context.genes[self.gene_key].code[self.pc];

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
        if self.pc >= context.genes[self.gene_key].code.len() {
            let top = self.call_stack.pop();
            match top {
                Some((gene_id, return_pc)) => {
                    // return to calling gene
                    // XXX must check for gene_id being valid
                    self.gene_key = context.cell.get_gene_key(gene_id).unwrap();
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

    pub fn execute_amount(&mut self, context: &mut ExecutionContext, amount: usize) {
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

    fn jump(&mut self, adjust: i32, context: &ExecutionContext) -> Option<()> {
        let new_pc: i32 = (self.pc as i32) + adjust;
        let gene = &context.genes[self.gene_key];
        if new_pc < 0 || new_pc >= (gene.code.len() as i32) {
            return None;
        }
        self.pc = new_pc as usize;
        Some(())
    }

    fn call(&mut self, gene_id: u32, context: &ExecutionContext) -> Option<()> {
        let gene = &context.genes[self.gene_key];
        context
            .cell
            .get_gene_key(gene_id)
            .and_then(|call_gene_key| {
                let return_pc = {
                    if self.pc >= gene.code.len() {
                        0
                    } else {
                        self.pc
                    }
                };
                self.call_stack.push((gene.id, return_pc));
                self.shrink_call_stack_on_overflow(context);
                self.gene_key = call_gene_key;
                self.pc = 0;
                Some(())
            })
    }

    fn read_gene(&mut self, gene_id: u32, index: u32, context: &ExecutionContext) -> Option<()> {
        context.cell.get_gene_key(gene_id).and_then(|gene_key| {
            let gene = &context.genes[gene_key];
            if index >= gene.code.len() as u32 {
                return None;
            }
            self.stack.push(gene.code[index as usize]);
            Some(())
        })
    }

    fn write_gene(
        &mut self,
        gene_id: u32,
        value: u32,
        context: &mut ExecutionContext,
    ) -> Option<()> {
        context.cell.get_gene_key(gene_id).and_then(|gene_key| {
            let gene = &mut context.genes[gene_key];
            gene.code.push(value);
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
    WriteGene = 0x010150,
}

impl<'a> ProcessorInstruction {
    pub fn execute(
        &self,
        processor: &mut Processor,
        context: &'a mut ExecutionContext,
    ) -> Option<()> {
        println!("Execute: {:?}", self);
        match self {
            ProcessorInstruction::JF => processor.stack.pop2().and_then(|(first, second)| {
                if !nr_to_bool(first) {
                    return Some(());
                }
                if second == 0 {
                    return Some(());
                }
                processor.jump(second as i32, context)
            }),
            ProcessorInstruction::JB => processor.stack.pop2().and_then(|(first, second)| {
                if !nr_to_bool(first) {
                    return Some(());
                }
                if second == 0 {
                    return Some(());
                }
                processor.jump(-(second as i32 + 1), context)
            }),
            ProcessorInstruction::Lookup => processor.stack.pop().and_then(|first| {
                processor
                    .stack
                    .push(context.cell.lookup_gene_id(context.genes, first));
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
            ProcessorInstruction::WriteGene => processor
                .stack
                .pop2()
                .and_then(|(first, second)| processor.write_gene(first, second, context)),
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
        processor: &mut Processor,
        context: &'a mut ExecutionContext,
    ) -> Option<()> {
        match self {
            Instruction::StackInstruction(instruction) => instruction.execute(&mut processor.stack),
            Instruction::ProcessorInstruction(instruction) => {
                instruction.execute(processor, context)
            }
        }
    }
    pub fn coordinates(&self) -> u32 {
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
    const WRITE_GENE_NR: u32 = ProcessorInstruction::WriteGene as u32 | INSTR_BIT;
    const SEED: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

    fn instruction_lookup<'a>() -> lookup::Lookup<Instruction> {
        let mut l = lookup::Lookup::<Instruction>::new();
        let mut add = |instruction: Instruction| {
            l.add(instruction.coordinates(), instruction)
                .expect("Cannot add!")
        };

        add(Instruction::StackInstruction(stack::Instruction::Add));

        add(Instruction::StackInstruction(stack::Instruction::Sub));
        add(Instruction::StackInstruction(stack::Instruction::Dup));
        add(Instruction::ProcessorInstruction(ProcessorInstruction::JF));
        add(Instruction::ProcessorInstruction(ProcessorInstruction::JB));
        add(Instruction::ProcessorInstruction(
            ProcessorInstruction::Lookup,
        ));
        add(Instruction::ProcessorInstruction(
            ProcessorInstruction::Call,
        ));
        add(Instruction::ProcessorInstruction(
            ProcessorInstruction::ReadGene,
        ));
        add(Instruction::ProcessorInstruction(
            ProcessorInstruction::WriteGene,
        ));
        return l;
    }
    #[test]
    fn test_processor_execute() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();

        let gene = Gene::new(0, &[3, 4, ADD_NR]);
        let gene_key = genes.insert(gene);
        let cell = Cell::new();
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 3);
        assert_eq!(g.stack, [7]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_processor_execute_multiple() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[3, 4, ADD_NR, 6, SUB_NR]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 5);

        assert_eq!(g.stack, [1]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_processor_execute_beyond_end() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[3, 4, ADD_NR]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 6);

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
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[3, 4, ADD_NR + 1, 6, SUB_NR - 1]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };
        g.execute_amount(&mut context, 5);

        assert_eq!(g.stack, [1]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_processor_execute_stack_underflow() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[4, ADD_NR]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 2);

        assert_eq!(g.stack, []);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_processor_execute_stack_overflow_numbers() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[1, 2, 3, 4, 5]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 4,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 5);

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
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();
        let gene = Gene::new(0, &[1, DUP_NR, DUP_NR, DUP_NR, DUP_NR]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 4,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 5);

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
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[1, 1, JF_NR, 66, 77]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 4);

        assert_eq!(g.stack, [77]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_jf2() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[1, 2, JF_NR, 66, 77, 88]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 4);

        assert_eq!(g.stack, [88]);
    }

    #[test]
    fn test_jf_too_far() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[1, 200, JF_NR, 66, 88]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 4);

        assert_eq!(g.stack, [66]);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_jf_false() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[0, 1, JF_NR, 66, 88]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 4);

        assert_eq!(g.stack, [66]);
    }

    #[test]
    fn test_jf_zero() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[1, 0, JF_NR, 66, 88]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 4);

        assert_eq!(g.stack, [66]);
    }

    #[test]
    fn test_jb() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();
        let gene = Gene::new(0, &[88, 1, 3, JB_NR, 66]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 5);

        assert_eq!(g.stack, [88, 88]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_jb_false() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[88, 0, 3, JB_NR, 66]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };
        g.execute_amount(&mut context, 5);

        assert_eq!(g.stack, [88, 66]);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_jb_1() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[88, 1, 1, JB_NR, 66]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);
        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 5);

        assert_eq!(g.stack, [88, 1]);
    }

    #[test]
    fn test_jb_zero() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[88, 1, 0, JB_NR, 66]);
        let gene_key = genes.insert(gene);
        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };

        g.execute_amount(&mut context, 5);

        assert_eq!(g.stack, [88, 66]);
    }

    #[test]
    fn test_jb_too_far() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let cell = Cell::new();

        let gene = Gene::new(0, &[88, 1, 100, JB_NR, 66]);
        let gene_key = genes.insert(gene);

        let mut g = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };
        g.execute_amount(&mut context, 5);

        assert_eq!(g.stack, [88, 66]);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_lookup() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        let gene1_id;
        {
            let gene1_key = cell.add_gene(&mut genes, &[3, 4, ADD_NR], &mut rng);
            gene1_id = genes[gene1_key].id;
        }

        let gene2_id;
        {
            let gene2_key = cell.add_gene(&mut genes, &[5, 3, LOOKUP_NR], &mut rng);
            gene2_id = genes[gene2_key].id;
        }

        let gene_key = cell.get_gene_key(gene2_id).unwrap();
        let mut p = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };
        p.execute_amount(&mut context, 3);
        assert_eq!(p.stack, [5, gene1_id]);
    }

    #[test]
    fn test_call_without_return() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        cell.add_gene(&mut genes, &[3, 4, ADD_NR], &mut rng);

        let gene2_id;
        {
            // 5 3
            // 5 <NR>
            // 5 3 4
            // 5 7
            let gene2_key = cell.add_gene(&mut genes, &[5, 3, LOOKUP_NR, CALL_NR], &mut rng);
            gene2_id = genes[gene2_key].id;
        }

        let gene_key = cell.get_gene_key(gene2_id).unwrap();
        let mut p = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };
        p.execute_amount(&mut context, 7);

        assert_eq!(p.stack, [5, 7]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_call_impossible_gene_id() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        let gene_key = cell.add_gene(&mut genes, &[5, CALL_NR, 1, 6, ADD_NR], &mut rng);
        let gene_id = genes[gene_key].id;

        let gene_key = cell.get_gene_key(gene_id).unwrap();
        let mut p = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };
        p.execute_amount(&mut context, 5);

        assert_eq!(p.stack, [7]);
        assert_eq!(p.failures, 1);
    }

    #[test]
    fn test_call_and_return() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        cell.add_gene(&mut genes, &[3, 4, ADD_NR], &mut rng);

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
            let gene2_key = cell.add_gene(&mut genes, &[5, 3, LOOKUP_NR, CALL_NR, 4], &mut rng);
            gene2_id = genes[gene2_key].id;
        }

        let gene_key = cell.get_gene_key(gene2_id).unwrap();
        let mut p = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };
        p.execute_amount(&mut context, 8);

        assert_eq!(p.stack, [5, 7, 4]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_call_at_end() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        cell.add_gene(&mut genes, &[3, 4, ADD_NR], &mut rng);

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
            let gene2_key = cell.add_gene(&mut genes, &[5, 3, LOOKUP_NR, CALL_NR], &mut rng);
            gene2_id = genes[gene2_key].id;
        }

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };
        let gene_key = cell.get_gene_key(gene2_id).unwrap();
        let mut p = Processor::new(gene_key);

        p.execute_amount(&mut context, 8);

        assert_eq!(p.stack, [5, 7, 5]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_call_stack_compaction() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        cell.add_gene(&mut genes, &[1, 2, LOOKUP_NR, CALL_NR], &mut rng);
        cell.add_gene(&mut genes, &[2, 3, LOOKUP_NR, CALL_NR], &mut rng);
        cell.add_gene(&mut genes, &[3, 4, 10, 20, ADD_NR, 40], &mut rng);
        let gene_key = cell.add_gene(&mut genes, &[0, 1, LOOKUP_NR, CALL_NR], &mut rng);
        let gene_id = genes[gene_key].id;

        let gene_key = cell.get_gene_key(gene_id).unwrap();
        let mut p = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 2,
            cell: &cell,
            genes: &mut genes,
        };
        p.execute_amount(&mut context, 17);

        assert_eq!(p.stack, [0, 1, 2, 3, 4, 30]);
        assert_eq!(p.call_stack.len(), 2);
        assert_eq!(p.failures, 1);
    }

    #[test]
    fn test_read_gene() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        cell.add_gene(&mut genes, &[3, 4, ADD_NR], &mut rng);

        let gene_id;
        {
            let gene_key = cell.add_gene(&mut genes, &[5, 3, LOOKUP_NR, 0, READ_GENE_NR], &mut rng);
            gene_id = genes[gene_key].id;
        }

        let gene_key = cell.get_gene_key(gene_id).unwrap();
        let mut p = Processor::new(gene_key);

        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };
        p.execute_amount(&mut context, 5);
        assert_eq!(p.stack, [5, 3]);
    }

    #[test]
    fn test_read_gene_other_index() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        cell.add_gene(&mut genes, &[3, 4, ADD_NR], &mut rng);

        let gene_id;
        {
            let gene_key = cell.add_gene(&mut genes, &[5, 3, LOOKUP_NR, 2, READ_GENE_NR], &mut rng);
            gene_id = genes[gene_key].id;
        }

        let gene_key = cell.get_gene_key(gene_id).unwrap();
        let mut p = Processor::new(gene_key);
        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };
        p.execute_amount(&mut context, 5);
        assert_eq!(p.stack, [5, ADD_NR]);
    }

    #[test]
    fn test_read_gene_beyond_end() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        cell.add_gene(&mut genes, &[3, 4, ADD_NR], &mut rng);

        let gene_id;
        {
            let gene_key =
                cell.add_gene(&mut genes, &[5, 3, LOOKUP_NR, 100, READ_GENE_NR], &mut rng);
            gene_id = genes[gene_key].id;
        }

        let gene_key = cell.get_gene_key(gene_id).unwrap();
        let mut p = Processor::new(gene_key);
        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };
        p.execute_amount(&mut context, 5);
        assert_eq!(p.stack, [5]);
    }

    #[test]
    fn test_write_gene() {
        let mut genes: DenseSlotMap<GeneKey, Gene> = DenseSlotMap::with_key();
        let mut cell = Cell::new();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        let gene_id1;
        {
            let gene_key = cell.add_gene(&mut genes, &[3, 4, ADD_NR], &mut rng);
            gene_id1 = genes[gene_key].id;
        }
        let gene_id2;
        {
            let gene_key =
                cell.add_gene(&mut genes, &[5, 3, LOOKUP_NR, 10, WRITE_GENE_NR], &mut rng);
            gene_id2 = genes[gene_key].id;
        }

        let gene2_key = cell.get_gene_key(gene_id2).unwrap();
        let mut p = Processor::new(gene2_key);
        let mut context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
            cell: &cell,
            genes: &mut genes,
        };
        p.execute_amount(&mut context, 5);
        let gene1_key = cell.get_gene_key(gene_id1).unwrap();
        assert_eq!(genes[gene1_key].code, [3, 4, ADD_NR, 10]);
    }
}
