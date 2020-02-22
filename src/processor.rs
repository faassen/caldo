use crate::cell::CellKey;
use crate::gene::GeneKey;
use crate::lookup;
use crate::stack;
use crate::stack::{nr_to_bool, Stack};
use crate::triplet::{Mode, Triplet};
use crate::world::{Entities, World};
use slotmap::new_key_type;

new_key_type! {pub struct ProcessorKey; }

pub struct Config {
    pub max_stack_size: usize,
    pub max_call_stack_size: usize,
    pub instruction_lookup: lookup::Lookup<Instruction>,
}

pub struct Processor {
    cell_key: CellKey,
    gene_key: GeneKey,
    pub stack: Vec<u32>,
    pub call_stack: Vec<(u32, usize)>,
    pc: usize,
    pub failures: u32,
}

impl Processor {
    pub fn new(cell_key: CellKey, gene_key: GeneKey) -> Processor {
        return Processor {
            cell_key: cell_key,
            gene_key: gene_key,
            stack: vec![],
            call_stack: vec![],
            pc: 0,
            failures: 0,
        };
    }

    pub fn execute(&mut self, entities: &mut Entities, config: &Config) {
        let value = entities.genes[self.gene_key].code[self.pc];

        // now increase pc
        self.pc += 1;

        let t = Triplet::from_int(value);
        match t.mode {
            Mode::Number => {
                // println!("number: {}", value);
                self.stack.push(value);
            }
            Mode::Instruction => {
                let instruction = config.instruction_lookup.find(value);
                // println!("value {:x?}, instruction: {:?}", value, instruction);
                let success = instruction.execute(self, entities, config);
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
        if self.pc >= entities.genes[self.gene_key].code.len() {
            let top = self.call_stack.pop();
            match top {
                Some((gene_id, return_pc)) => {
                    // return to calling gene
                    // XXX must check for gene_id being valid!
                    // XXX must check that gene is in the same cell!
                    self.gene_key = entities.get_gene_key(self.cell_key, gene_id).unwrap();
                    self.pc = return_pc;
                }
                None => {
                    // go back to start
                    self.pc = 0;
                }
            }
        }
        self.shrink_stack_on_overflow(config);
    }

    pub fn execute_amount(&mut self, world: &mut World, amount: usize) {
        (0..amount).for_each(|_| self.execute(&mut world.entities, &world.config))
    }

    pub fn shrink_stack_on_overflow(&mut self, config: &Config) {
        if self.stack.len() <= config.max_stack_size {
            return;
        }
        self.failures += 1;
        self.stack
            .splice(..config.max_stack_size / 2, [].iter().cloned());
    }

    pub fn shrink_call_stack_on_overflow(&mut self, config: &Config) {
        if self.call_stack.len() <= config.max_call_stack_size {
            return;
        }
        self.failures += 1;
        self.call_stack
            .splice(..config.max_call_stack_size / 2, [].iter().cloned());
    }

    fn jump(&mut self, adjust: i32, entities: &Entities) -> Option<()> {
        let new_pc: i32 = (self.pc as i32) + adjust;
        let gene = &entities.genes[self.gene_key];
        if new_pc < 0 || new_pc >= (gene.code.len() as i32) {
            return None;
        }
        self.pc = new_pc as usize;
        Some(())
    }

    fn call(&mut self, gene_id: u32, entities: &Entities, config: &Config) -> Option<()> {
        let gene = &entities.genes[self.gene_key];
        // XXX need to check whether gene is in the same cell
        entities
            .get_gene_key(self.cell_key, gene_id)
            .and_then(|call_gene_key| {
                let return_pc = {
                    if self.pc >= gene.code.len() {
                        0
                    } else {
                        self.pc
                    }
                };
                self.call_stack.push((gene.id, return_pc));
                self.shrink_call_stack_on_overflow(config);
                self.gene_key = call_gene_key;
                self.pc = 0;
                Some(())
            })
    }

    fn gene_read(&mut self, gene_id: u32, index: u32, entities: &Entities) -> Option<()> {
        entities
            .get_gene_key(self.cell_key, gene_id)
            .and_then(|gene_key| {
                let gene = &entities.genes[gene_key];
                if index >= gene.code.len() as u32 {
                    return None;
                }
                self.stack.push(gene.code[index as usize]);
                Some(())
            })
    }

    fn gene_write(&mut self, gene_id: u32, value: u32, entities: &mut Entities) -> Option<()> {
        entities
            .get_gene_key(self.cell_key, gene_id)
            .and_then(|gene_key| {
                let gene = &mut entities.genes[gene_key];
                gene.code.push(value);
                Some(())
            })
    }

    fn start_proc(&mut self, gene_id: u32, index: u32, entities: &mut Entities) -> Option<()> {
        entities
            .get_gene_key(self.cell_key, gene_id)
            .and_then(|gene_key| {
                let gene = &entities.genes[gene_key];
                if index >= gene.code.len() as u32 {
                    return None;
                }
                // XXX not finished yet
                // world.add_processor(gene_key, index);
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
    GeneRead = 0x010140,
    GeneWrite = 0x010150,
    StartProc = 0x010160,
}

impl<'a> ProcessorInstruction {
    pub fn execute(
        &self,
        processor: &mut Processor,
        world: &mut Entities,
        config: &'a Config,
    ) -> Option<()> {
        match self {
            ProcessorInstruction::JF => processor.stack.pop2().and_then(|(first, second)| {
                if !nr_to_bool(first) {
                    return Some(());
                }
                if second == 0 {
                    return Some(());
                }
                processor.jump(second as i32, world)
            }),
            ProcessorInstruction::JB => processor.stack.pop2().and_then(|(first, second)| {
                if !nr_to_bool(first) {
                    return Some(());
                }
                if second == 0 {
                    return Some(());
                }
                processor.jump(-(second as i32 + 1), world)
            }),
            ProcessorInstruction::Lookup => processor.stack.pop().and_then(|first| {
                processor
                    .stack
                    .push(world.cells[processor.cell_key].lookup_gene_id(&world.genes, first));
                Some(())
            }),
            ProcessorInstruction::Call => processor
                .stack
                .pop()
                .and_then(|first| processor.call(first, world, config)),
            ProcessorInstruction::GeneRead => processor
                .stack
                .pop2()
                .and_then(|(first, second)| processor.gene_read(first, second, world)),
            ProcessorInstruction::GeneWrite => processor
                .stack
                .pop2()
                .and_then(|(first, second)| processor.gene_write(first, second, world)),
            ProcessorInstruction::StartProc => processor
                .stack
                .pop2()
                .and_then(|(first, second)| processor.start_proc(first, second, world)),
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
        world: &mut Entities,
        config: &'a Config,
    ) -> Option<()> {
        match self {
            Instruction::StackInstruction(instruction) => instruction.execute(&mut processor.stack),
            Instruction::ProcessorInstruction(instruction) => {
                instruction.execute(processor, world, config)
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
    use crate::cell::Cell;
    use crate::gene::Gene;
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
    const GENE_READ_NR: u32 = ProcessorInstruction::GeneRead as u32 | INSTR_BIT;
    const GENE_WRITE_NR: u32 = ProcessorInstruction::GeneWrite as u32 | INSTR_BIT;
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
            ProcessorInstruction::GeneRead,
        ));
        add(Instruction::ProcessorInstruction(
            ProcessorInstruction::GeneWrite,
        ));
        return l;
    }
    #[test]
    fn test_processor_execute() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[3, 4, ADD_NR]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 3);
        assert_eq!(p.stack, [7]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_processor_execute_multiple() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[3, 4, ADD_NR, 6, SUB_NR]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);

        assert_eq!(p.stack, [1]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_processor_execute_beyond_end() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[3, 4, ADD_NR]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 6);

        // 3
        // 4
        // 7
        // 7 3
        // 7 3 4
        // 7 7

        assert_eq!(p.stack, [7, 7]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_processor_execute_nearby() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();

        let gene_key = world.create_gene(&[3, 4, ADD_NR + 1, 6, SUB_NR - 1]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);

        assert_eq!(p.stack, [1]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_processor_execute_stack_underflow() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[4, ADD_NR]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 2);

        assert_eq!(p.stack, []);
        assert_eq!(p.failures, 1);
    }

    #[test]
    fn test_processor_execute_stack_overflow_numbers() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 4,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[1, 2, 3, 4, 5]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);

        // 1
        // 1 2
        // 1 2 3
        // 1 2 3 4
        // 3 4 5

        assert_eq!(p.stack, [3, 4, 5]);
        assert_eq!(p.failures, 1);
    }

    #[test]
    fn test_processor_execute_stack_overflow_instructions() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 4,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[1, DUP_NR, DUP_NR, DUP_NR, DUP_NR]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);

        // 1
        // 1 1
        // 1 1 1
        // 1 1 1 1
        // 1 1 1 1 1
        assert_eq!(p.stack, [1, 1, 1]);
        assert_eq!(p.failures, 1);
    }

    #[test]
    fn test_jf() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();

        let gene_key = world.create_gene(&[1, 1, JF_NR, 66, 77]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 4);

        assert_eq!(p.stack, [77]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_jf2() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[1, 2, JF_NR, 66, 77, 88]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 4);

        assert_eq!(p.stack, [88]);
    }

    #[test]
    fn test_jf_too_far() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[1, 200, JF_NR, 66, 88]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 4);

        assert_eq!(p.stack, [66]);
        assert_eq!(p.failures, 1);
    }

    #[test]
    fn test_jf_false() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[0, 1, JF_NR, 66, 88]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 4);

        assert_eq!(p.stack, [66]);
    }

    #[test]
    fn test_jf_zero() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[1, 0, JF_NR, 66, 88]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 4);

        assert_eq!(p.stack, [66]);
    }

    #[test]
    fn test_jb() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[88, 1, 3, JB_NR, 66]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);

        assert_eq!(p.stack, [88, 88]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_jb_false() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[88, 0, 3, JB_NR, 66]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);

        assert_eq!(p.stack, [88, 66]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_jb_1() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[88, 1, 1, JB_NR, 66]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);

        assert_eq!(p.stack, [88, 1]);
    }

    #[test]
    fn test_jb_zero() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[88, 1, 0, JB_NR, 66]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);

        assert_eq!(p.stack, [88, 66]);
    }

    #[test]
    fn test_jb_too_far() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let gene_key = world.create_gene(&[88, 1, 100, JB_NR, 66]);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);

        assert_eq!(p.stack, [88, 66]);
        assert_eq!(p.failures, 1);
    }

    #[test]
    fn test_lookup() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        let gene1_key = world.create_gene_in_cell(cell_key, &[3, 4, ADD_NR], &mut rng);
        let gene1_id = world.entities.genes[gene1_key].id;
        let gene2_key = world.create_gene_in_cell(cell_key, &[5, 3, LOOKUP_NR], &mut rng);

        let mut p = Processor::new(cell_key, gene2_key);

        p.execute_amount(&mut world, 3);
        assert_eq!(p.stack, [5, gene1_id]);
    }

    #[test]
    fn test_lookup_in_other_cell_fails() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell1_key = world.create_cell();
        let cell2_key = world.create_cell();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        // put this gene in another cell, so lookup should find itself
        world.create_gene_in_cell(cell2_key, &[3, 4, ADD_NR], &mut rng);
        let gene2_key = world.create_gene_in_cell(cell1_key, &[5, 3, LOOKUP_NR], &mut rng);
        let gene2_id = world.entities.genes[gene2_key].id;
        // lookup finds the gene itself
        let mut p = Processor::new(cell1_key, gene2_key);

        p.execute_amount(&mut world, 3);
        assert_eq!(p.stack, [5, gene2_id]);
    }

    #[test]
    fn test_call_without_return() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        world.create_gene_in_cell(cell_key, &[3, 4, ADD_NR], &mut rng);
        // 5 3
        // 5 <NR>
        // 5 3 4
        // 5 7
        let gene2_key = world.create_gene_in_cell(cell_key, &[5, 3, LOOKUP_NR, CALL_NR], &mut rng);

        let mut p = Processor::new(cell_key, gene2_key);

        p.execute_amount(&mut world, 7);

        assert_eq!(p.stack, [5, 7]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_call_impossible_gene_id() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        let other_cell_key = world.create_cell();
        let other_cell_gene_key = world.create_gene_in_cell(other_cell_key, &[6, 7, 8], &mut rng);
        let other_cell_gene_id = world.entities.genes[other_cell_gene_key].id;
        let gene_key = world.create_gene_in_cell(
            cell_key,
            &[other_cell_gene_id, CALL_NR, 1, 6, ADD_NR],
            &mut rng,
        );
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);

        assert_eq!(p.stack, [7]);
        assert_eq!(p.failures, 1);
    }

    #[test]
    fn test_call_gene_id_in_another_cell() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        let gene_key = world.create_gene_in_cell(cell_key, &[5, CALL_NR, 1, 6, ADD_NR], &mut rng);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);

        assert_eq!(p.stack, [7]);
        assert_eq!(p.failures, 1);
    }

    #[test]
    fn test_call_and_return() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        world.create_gene_in_cell(cell_key, &[3, 4, ADD_NR], &mut rng);

        // 5
        // 5 3
        // 5 <NR>
        // 5
        // 5 3
        // 5 3 4
        // 5 7
        // 5 7 4
        let gene2_key =
            world.create_gene_in_cell(cell_key, &[5, 3, LOOKUP_NR, CALL_NR, 4], &mut rng);

        let mut p = Processor::new(cell_key, gene2_key);

        p.execute_amount(&mut world, 8);

        assert_eq!(p.stack, [5, 7, 4]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_call_at_end() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        world.create_gene_in_cell(cell_key, &[3, 4, ADD_NR], &mut rng);
        // 5
        // 5 3
        // 5 <NR>
        // 5
        // 5 3
        // 5 3 4
        // 5 7
        // should wrap again to start
        // 5 7 5
        let gene2_key = world.create_gene_in_cell(cell_key, &[5, 3, LOOKUP_NR, CALL_NR], &mut rng);

        let mut p = Processor::new(cell_key, gene2_key);

        p.execute_amount(&mut world, 8);

        assert_eq!(p.stack, [5, 7, 5]);
        assert_eq!(p.failures, 0);
    }

    #[test]
    fn test_call_stack_compaction() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 2,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        world.create_gene_in_cell(cell_key, &[1, 2, LOOKUP_NR, CALL_NR], &mut rng);
        world.create_gene_in_cell(cell_key, &[2, 3, LOOKUP_NR, CALL_NR], &mut rng);
        world.create_gene_in_cell(cell_key, &[3, 4, 10, 20, ADD_NR, 40], &mut rng);
        let gene_key = world.create_gene_in_cell(cell_key, &[0, 1, LOOKUP_NR, CALL_NR], &mut rng);

        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 17);

        assert_eq!(p.stack, [0, 1, 2, 3, 4, 30]);
        assert_eq!(p.call_stack.len(), 2);
        assert_eq!(p.failures, 1);
    }

    #[test]
    fn test_read_gene() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        world.create_gene_in_cell(cell_key, &[3, 4, ADD_NR], &mut rng);

        let gene_key =
            world.create_gene_in_cell(cell_key, &[5, 3, LOOKUP_NR, 0, GENE_READ_NR], &mut rng);

        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);
        assert_eq!(p.stack, [5, 3]);
    }

    #[test]
    fn test_read_gene_other_index() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        world.create_gene_in_cell(cell_key, &[3, 4, ADD_NR], &mut rng);

        let gene_key =
            world.create_gene_in_cell(cell_key, &[5, 3, LOOKUP_NR, 2, GENE_READ_NR], &mut rng);

        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);
        assert_eq!(p.stack, [5, ADD_NR]);
    }

    #[test]
    fn test_read_gene_beyond_end() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);

        world.create_gene_in_cell(cell_key, &[3, 4, ADD_NR], &mut rng);

        let gene_key =
            world.create_gene_in_cell(cell_key, &[5, 3, LOOKUP_NR, 100, GENE_READ_NR], &mut rng);
        let mut p = Processor::new(cell_key, gene_key);

        p.execute_amount(&mut world, 5);
        assert_eq!(p.stack, [5]);
    }

    #[test]
    fn test_write_gene() {
        let config = Config {
            instruction_lookup: instruction_lookup(),
            max_stack_size: 1000,
            max_call_stack_size: 1000,
        };
        let mut world = World::new(config);
        let cell_key = world.create_cell();
        let mut rng = rand_pcg::Pcg32::from_seed(SEED);
        let gene1_key = world.create_gene_in_cell(cell_key, &[3, 4, ADD_NR], &mut rng);
        let gene2_key =
            world.create_gene_in_cell(cell_key, &[5, 3, LOOKUP_NR, 10, GENE_WRITE_NR], &mut rng);

        let mut p = Processor::new(cell_key, gene2_key);

        p.execute_amount(&mut world, 5);

        assert_eq!(world.entities.genes[gene1_key].code, [3, 4, ADD_NR, 10]);
    }
}
