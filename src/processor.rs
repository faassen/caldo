use rand::Rng;

use crate::cell::CellKey;
use crate::gene::GeneKey;
use crate::lookup;
use crate::stack;
use crate::stack::{nr_to_bool, Stack};
use crate::triplet::{Mode, Triplet};
use crate::world::Entities;
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

    pub fn execute<R: Rng>(
        &mut self,
        entities: &Entities,
        config: &Config,
        rng: &mut R,
    ) -> Option<Action> {
        let value = entities.genes[self.gene_key].code[self.pc];

        // now increase pc
        self.pc += 1;

        let t = Triplet::from_int(value);
        let action: Option<Action> = match t.mode {
            Mode::Number => {
                // println!("number: {}", value);
                self.stack.push(value);
                None
            }
            Mode::Instruction => {
                let instruction = config.instruction_lookup.find(value);
                // println!("value {:x?}, instruction: {:?}", value, instruction);
                let action = instruction.execute(self, entities, config, rng);
                if action.is_none() {
                    self.failures += 1;
                }
                action
            }
            Mode::Call => None,
            Mode::Noop => None,
        };

        // at the end
        if self.pc >= entities.genes[self.gene_key].code.len() {
            let top = self.call_stack.pop();
            match top {
                Some((gene_id, return_pc)) => {
                    // return to calling gene
                    // XXX must check for gene_id being valid! unwrap isn't safe
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
        action
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

    fn jump(&mut self, adjust: i32, entities: &Entities) -> Option<Action> {
        let new_pc: i32 = (self.pc as i32) + adjust;
        let gene = &entities.genes[self.gene_key];
        if new_pc < 0 || new_pc >= (gene.code.len() as i32) {
            return None;
        }
        self.pc = new_pc as usize;
        Some(Action::Noop)
    }

    fn call(&mut self, gene_id: u32, entities: &Entities, config: &Config) -> Option<Action> {
        let gene = &entities.genes[self.gene_key];
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
                Some(Action::Noop)
            })
    }

    fn gene_read(&mut self, gene_id: u32, index: u32, entities: &Entities) -> Option<Action> {
        entities
            .get_gene_key(self.cell_key, gene_id)
            .and_then(|gene_key| {
                let gene = &entities.genes[gene_key];
                if index >= gene.code.len() as u32 {
                    return None;
                }
                self.stack.push(gene.code[index as usize]);
                Some(Action::Noop)
            })
    }

    fn gene_write(&self, gene_id: u32, value: u32, entities: &Entities) -> Option<Action> {
        entities
            .get_gene_key(self.cell_key, gene_id)
            .and_then(|gene_key| Some(Action::GeneWrite(gene_key, value)))
    }

    // fn proc_start(&self, gene_id: u32, index: u32, entities: &Entities) -> Option<()> {
    //     entities
    //         .get_gene_key(self.cell_key, gene_id)
    //         .and_then(|gene_key| {
    //             let gene = &entities.genes[gene_key];
    //             if index >= gene.code.len() as u32 {
    //                 return None;
    //             }
    //             // XXX not finished yet
    //             // world.add_processor(gene_key, index);
    //             Some(())
    //         })
    // }
}

pub enum Action {
    Noop,
    GeneWrite(GeneKey, u32),
    GeneCreate(CellKey, u32),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ProcessorInstruction {
    JF = 0x010100,
    JB = 0x010110,
    Lookup = 0x010120,
    Call = 0x010130,
    GeneRead = 0x010140,
    GeneWrite = 0x010150,
    GeneCreate = 0x010160,
    // ProcStart = 0x010160,
}

impl<'a> ProcessorInstruction {
    pub fn execute<R: Rng>(
        &self,
        processor: &mut Processor,
        entities: &Entities,
        config: &'a Config,
        rng: &mut R,
    ) -> Option<Action> {
        match self {
            ProcessorInstruction::JF => processor.stack.pop2().and_then(|(first, second)| {
                if !nr_to_bool(first) {
                    return Some(Action::Noop);
                }
                if second == 0 {
                    return Some(Action::Noop);
                }
                processor.jump(second as i32, entities)
            }),
            ProcessorInstruction::JB => processor.stack.pop2().and_then(|(first, second)| {
                if !nr_to_bool(first) {
                    return Some(Action::Noop);
                }
                if second == 0 {
                    return Some(Action::Noop);
                }
                processor.jump(-(second as i32 + 1), entities)
            }),
            ProcessorInstruction::Lookup => processor.stack.pop().and_then(|first| {
                processor.stack.push(
                    entities.cells[processor.cell_key].lookup_gene_id(&entities.genes, first),
                );
                Some(Action::Noop)
            }),
            ProcessorInstruction::Call => processor
                .stack
                .pop()
                .and_then(|first| processor.call(first, entities, config)),
            ProcessorInstruction::GeneRead => processor
                .stack
                .pop2()
                .and_then(|(first, second)| processor.gene_read(first, second, entities)),
            ProcessorInstruction::GeneWrite => processor
                .stack
                .pop2()
                .and_then(|(first, second)| processor.gene_write(first, second, entities)),
            ProcessorInstruction::GeneCreate => {
                let id = entities.create_gene_id(rng);
                processor.stack.push(id);
                Some(Action::GeneCreate(processor.cell_key, id))
            }
                // ProcessorInstruction::ProcStart => processor
            //     .stack
            //     .pop2()
            //     .and_then(|(first, second)| processor.proc_start(first, second, entities)),
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
    pub fn execute<R: Rng>(
        &self,
        processor: &mut Processor,
        entities: &Entities,
        config: &'a Config,
        rng: &mut R,
    ) -> Option<Action> {
        match self {
            Instruction::StackInstruction(instruction) => instruction
                .execute(&mut processor.stack)
                .map(|_| Action::Noop),
            Instruction::ProcessorInstruction(instruction) => {
                instruction.execute(processor, entities, config, rng)
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
