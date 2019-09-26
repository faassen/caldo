use crate::triplet;
use kdtree::distance::squared_euclidean;
use kdtree::ErrorKind;
use kdtree::KdTree;

struct Gene {
    code: Vec<u32>,
    stack: Vec<u32>,
    pc: usize,
}

impl Gene {
    fn execute(&mut self, lookup: &InstructionLookup)  {
        let value = self.code[self.pc];
        let t = triplet::Triplet::from_int(value);
        if !t.instruction {
            self.stack.push(value);
            return;
        }
        let instruction = lookup.find(t);
    }
}

struct Instruction {}

struct InstructionLookup<'a> {
    tree: KdTree<f32, &'a Instruction, [f32; 3]>,
}

impl<'a> InstructionLookup<'a> {
    fn add(&mut self, triplet: triplet::Triplet, instruction: &'a Instruction) {
        self.tree.add(triplet.coordinates(), instruction);
    }

    fn find(&self, t: triplet::Triplet) -> &Instruction {
        let v = self
            .tree
            .nearest(&t.coordinates(), 1, &squared_euclidean)
            .unwrap();
        return v[0].1;
    }
}
