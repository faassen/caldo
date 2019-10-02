use crate::triplet::Triplet;
use kdtree::distance::squared_euclidean;
use kdtree::ErrorKind;
use kdtree::KdTree;

pub struct Gene {
    code: Vec<u32>,
    stack: Vec<u32>,
    pc: usize,
    failures: u32,
}

impl Gene {
    pub fn execute(&mut self, lookup: &InstructionLookup) {
        let value = self.code[self.pc];
        let t = Triplet::from_int(value);
        if !t.instruction {
            self.stack.push(value);
            return;
        }
        let instruction = lookup.find(t);
        let success = instruction.execute(&mut self.stack);
        match success {
            Some(_) => {
                self.failures += 1;
            }
            None => {}
        }
        self.pc += 1;
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    Add,
    Sub,
    Mul,
    Div,
}

trait Stack<T> {
    fn pop2(&mut self) -> Option<(T, T)>;
}

impl<T> Stack<T> for Vec<T> {
    fn pop2(&mut self) -> Option<(T, T)> {
        return self
            .pop()
            .and_then(|first| self.pop().and_then(|second| Some((second, first))));
    }
}

trait OpStack<T, F>
where
    F: FnOnce(T, T) -> Option<T>,
{
    fn op2(&mut self, op: F) -> Option<()>;
}

impl<T, F> OpStack<T, F> for Vec<T>
where
    F: FnOnce(T, T) -> Option<T>,
{
    fn op2(&mut self, op: F) -> Option<()> {
        return self.pop2().and_then(|(x, y)| op(x, y)).and_then(|result| {
            self.push(result);
            return Some(());
        });
    }
}

impl Instruction {
    fn execute(&self, stack: &mut Vec<u32>) -> Option<()> {
        match self {
            Instruction::Add => stack.op2(|first, second| first.checked_add(second)),
            Instruction::Sub => stack.op2(|first, second| first.checked_sub(second)),
            Instruction::Mul => stack.op2(|first, second| first.checked_mul(second)),
            Instruction::Div => stack.op2(|first, second| first.checked_div(second)),
        }
    }
}

pub struct InstructionLookup {
    tree: KdTree<f32, Instruction, [f32; 3]>,
}

#[derive(Debug, Clone, Copy)]
pub struct InstructionLookupError {}

pub type InstructionLookupAddResult = Result<(), InstructionLookupError>;

impl From<ErrorKind> for InstructionLookupError {
    fn from(_error: ErrorKind) -> Self {
        InstructionLookupError {}
    }
}

impl InstructionLookup {
    pub fn new() -> InstructionLookup {
        return InstructionLookup {
            tree: KdTree::new(3),
        };
    }

    pub fn add(
        &mut self,
        triplet: Triplet,
        instruction: Instruction,
    ) -> InstructionLookupAddResult {
        self.tree.add(triplet.coordinates(), instruction)?;
        return Ok(());
    }

    pub fn find(&self, t: Triplet) -> Instruction {
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

    #[test]
    fn test_add_execute() {
        let mut s = vec![4u32, 3u32];
        let b = Instruction::Add.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s.len(), 1);
        assert_eq!(s[0], 7);
    }

    #[test]
    fn test_add_execute_overflow() {
        let mut s = vec![u32::max_value(), 1u32];
        let b = Instruction::Add.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_add_execute_stack_underflow_empty_stack() {
        let mut s = vec![];
        let b = Instruction::Add.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_add_execute_stack_underflow_too_little_on_stack() {
        let mut s = vec![4u32];
        let b = Instruction::Add.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_sub_execute() {
        let mut s = vec![4u32, 3u32];
        let b = Instruction::Sub.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s.len(), 1);
        assert_eq!(s[0], 1);
    }

    #[test]
    fn test_sub_execute_underflow() {
        let mut s = vec![4u32, 5u32];
        let b = Instruction::Sub.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_mul_execute() {
        let mut s = vec![4u32, 3u32];
        let b = Instruction::Mul.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s.len(), 1);
        assert_eq!(s[0], 12);
    }

    #[test]
    fn test_div_execute() {
        let mut s = vec![12u32, 3u32];
        let b = Instruction::Div.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s.len(), 1);
        assert_eq!(s[0], 4);
    }

}
