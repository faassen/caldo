use crate::triplet::Triplet;
use kdtree::distance::squared_euclidean;
use kdtree::ErrorKind;
use kdtree::KdTree;

struct Gene {
    code: Vec<u32>,
    stack: Vec<u32>,
    pc: usize,
}

impl Gene {
    fn execute(&mut self, lookup: &InstructionLookup) {
        let value = self.code[self.pc];
        let t = Triplet::from_int(value);
        if !t.instruction {
            self.stack.push(value);
            return;
        }
        let instruction = lookup.find(t);
        instruction.execute(&mut self.stack);
        self.pc += 1;
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Instruction {
    Add,
    Sub,
}

impl Instruction {
    fn execute(&self, stack: &mut Vec<u32>) {
        match self {
            Instruction::Add => {
                let v1 = stack.pop();
                let v2 = stack.pop();
                match v1 {
                    Some(v) => {}
                    None => {}
                }
            }
            Instruction::Sub => {}
        }
    }
}

struct InstructionLookup {
    tree: KdTree<f32, Instruction, [f32; 3]>,
}

#[derive(Debug, Clone, Copy)]
struct InstructionLookupError {}

type InstructionLookupAddResult = Result<(), InstructionLookupError>;

impl From<ErrorKind> for InstructionLookupError {
    fn from(error: ErrorKind) -> Self {
        InstructionLookupError {}
    }
}

impl InstructionLookup {
    fn new() -> InstructionLookup {
        return InstructionLookup {
            tree: KdTree::new(3),
        };
    }

    fn add(&mut self, triplet: Triplet, instruction: Instruction) -> InstructionLookupAddResult {
        self.tree.add(triplet.coordinates(), instruction)?;
        return Ok(());
    }

    fn find(&self, t: Triplet) -> Instruction {
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

}
