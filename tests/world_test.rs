use rand::SeedableRng;

use caldo::lookup;
use caldo::processor::{Config, Instruction, ProcessorInstruction};
use caldo::stack;
use caldo::world::World;

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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(3, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(5, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(6, &mut rng);

    // 3
    // 4
    // 7
    // 7 3
    // 7 3 4
    // 7 7
    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(5, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(2, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(5, &mut rng);

    // 1
    // 1 2
    // 1 2 3
    // 1 2 3 4
    // 3 4 5
    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(5, &mut rng);

    // 1
    // 1 1
    // 1 1 1
    // 1 1 1 1
    // 1 1 1 1 1
    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(4, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(4, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(4, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(4, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(4, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(5, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(5, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(5, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(5, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);
    let mut rng = rand_pcg::Pcg32::from_seed(SEED);

    world.execute_amount(5, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene2_key);

    world.execute_amount(3, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell1_key, gene2_key);

    world.execute_amount(3, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene2_key);

    world.execute_amount(7, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);

    world.execute_amount(5, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);

    world.execute_amount(5, &mut rng);

    let p = &world.processors[processor_key];
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
    let gene2_key = world.create_gene_in_cell(cell_key, &[5, 3, LOOKUP_NR, CALL_NR, 4], &mut rng);
    let processor_key = world.create_processor(cell_key, gene2_key);

    world.execute_amount(8, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene2_key);

    world.execute_amount(8, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);

    world.execute_amount(17, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);

    world.execute_amount(5, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);

    world.execute_amount(5, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene_key);

    world.execute_amount(5, &mut rng);

    let p = &world.processors[processor_key];
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
    let processor_key = world.create_processor(cell_key, gene2_key);

    world.execute_amount(5, &mut rng);

    assert_eq!(world.entities.genes[gene1_key].code, [3, 4, ADD_NR, 10]);
}
