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
    }
}

#[derive(Debug, PartialEq)]
struct Instruction {
    name: String,
}

struct InstructionLookup<'a> {
    tree: KdTree<f32, &'a Instruction, [f32; 3]>,
}

impl<'a> InstructionLookup<'a> {
    fn new() -> InstructionLookup<'a> {
        return InstructionLookup {
            tree: KdTree::new(3),
        };
    }

    fn add(&mut self, triplet: Triplet, instruction: &'a Instruction) {
        self.tree.add(triplet.coordinates(), instruction);
    }
    fn find(&self, t: Triplet) -> &Instruction {
        let v = self
            .tree
            .nearest(&t.coordinates(), 1, &squared_euclidean)
            .unwrap();
        return v[0].1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_lookup_identify() {
        let mut l = InstructionLookup::new();
        let t = Triplet::from_int(0x010203);
        let i = Instruction {
            name: "I".to_string(),
        };
        l.add(t, &i);
        assert_eq!(l.find(t), &i);
    }

    #[test]
    fn test_instruction_lookup_near() {
        let mut l = InstructionLookup::new();
        let t1 = Triplet::from_int(0x010203);
        let t2 = Triplet::from_int(0xFFFFFF);
        let tlookup = Triplet::from_int(0x010402);

        let i1 = Instruction {
            name: "I1".to_string(),
        };
        let i2 = Instruction {
            name: "I2".to_string(),
        };
        l.add(t1, &i1);
        l.add(t2, &i2);
        assert_eq!(l.find(tlookup), &i1);
    }

}
