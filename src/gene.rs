use crate::triplet::{Mode, Triplet};
use kdtree::distance::squared_euclidean;
use kdtree::ErrorKind;
use kdtree::KdTree;

const TRUE: u32 = 0xFFFFFFFF;
const FALSE: u32 = 0;

pub struct Gene<'a> {
    code: &'a [u32],
    stack: Vec<u32>,
    pc: usize,
    failures: u32,
}

pub struct ExecutionContext<'a> {
    max_stack_size: usize,
    instruction_lookup: &'a InstructionLookup,
}

impl<'a> Gene<'a> {
    pub fn new(code: &[u32]) -> Gene {
        return Gene {
            code: code,
            stack: vec![],
            pc: 0,
            failures: 0,
        };
    }

    pub fn execute(&mut self, context: &ExecutionContext) {
        let value = self.code[self.pc];
        self.pc += 1;
        if self.pc >= self.code.len() {
            self.pc = 0;
        }
        let t = Triplet::from_int(value);
        match t.mode {
            Mode::Number => {
                self.stack.push(value);
            }
            Mode::Instruction => {
                let instruction = context.instruction_lookup.find(t);
                let success = instruction.execute(&mut self.stack);
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

    pub fn shrink_stack_on_overflow(&mut self, context: &ExecutionContext) {
        if self.stack.len() <= context.max_stack_size {
            return;
        }
        self.failures += 1;
        self.stack
            .splice(..context.max_stack_size / 2, [].iter().cloned());
    }
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Instruction {
    Add,
    Sub,
    Mul,
    Div,
    Dup,
    Drop,
    Swap,
    Over,
    Rot,
    Eq,
    Ne,
    Gt,
    Lt,
    And,
    Or,
    Not,
}

// conditional execution, if

// no looping?

fn bool_to_nr(b: bool) -> u32 {
    if b {
        TRUE
    } else {
        FALSE
    }
}

fn nr_to_bool(nr: u32) -> bool {
    nr != 0
}

impl Instruction {
    fn execute(&self, stack: &mut Vec<u32>) -> Option<()> {
        match self {
            Instruction::Add => stack.op2(|first, second| first.checked_add(second)),
            Instruction::Sub => stack.op2(|first, second| first.checked_sub(second)),
            Instruction::Mul => stack.op2(|first, second| first.checked_mul(second)),
            Instruction::Div => stack.op2(|first, second| first.checked_div(second)),
            Instruction::Eq => stack.op2(|first, second| Some(bool_to_nr(first == second))),
            Instruction::Ne => stack.op2(|first, second| Some(bool_to_nr(first != second))),
            Instruction::Gt => stack.op2(|first, second| Some(bool_to_nr(first > second))),
            Instruction::Lt => stack.op2(|first, second| Some(bool_to_nr(first < second))),
            Instruction::And => {
                stack.op2(|first, second| Some(bool_to_nr(nr_to_bool(first) && nr_to_bool(second))))
            }
            Instruction::Or => {
                stack.op2(|first, second| Some(bool_to_nr(nr_to_bool(first) || nr_to_bool(second))))
            }
            Instruction::Not => stack.pop().and_then(|v| {
                stack.push(bool_to_nr(!nr_to_bool(v)));
                return Some(());
            }),
            Instruction::Dup => stack.pop().and_then(|v| {
                stack.push(v);
                stack.push(v);
                return Some(());
            }),
            Instruction::Drop => stack.pop().and_then(|_v| return Some(())),
            Instruction::Swap => stack.pop2().and_then(|(x, y)| {
                stack.push(y);
                stack.push(x);
                return Some(());
            }),
            Instruction::Over => {
                if stack.len() < 2 {
                    stack.clear();
                    return None;
                }
                stack.push(stack[stack.len() - 2]);
                return Some(());
            }
            Instruction::Rot => {
                if stack.len() < 3 {
                    stack.clear();
                    return None;
                }
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                let c = stack.pop().unwrap();

                stack.push(b);
                stack.push(a);
                stack.push(c);
                return Some(());
            }
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

    const ADD_NR: u32 = 0x01010203;
    const SUB_NR: u32 = 0x01030201;
    const DUP_NR: u32 = 0x01070707;

    fn instruction_lookup() -> InstructionLookup {
        let mut l = InstructionLookup::new();
        let add_triplet = Triplet::from_int(ADD_NR);
        let sub_triplet = Triplet::from_int(SUB_NR);
        let dup_triplet = Triplet::from_int(DUP_NR);
        l.add(add_triplet, Instruction::Add).expect("cannot add");
        l.add(sub_triplet, Instruction::Sub).expect("cannot add");
        l.add(dup_triplet, Instruction::Dup).expect("cannot add");
        return l;
    }
    #[test]
    fn test_gene_execute() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[3, 4, ADD_NR]);

        g.execute(&context);
        g.execute(&context);
        g.execute(&context);

        assert_eq!(g.stack.len(), 1);
        assert_eq!(g.stack[0], 7);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_gene_execute_multiple() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[3, 4, ADD_NR, 6, SUB_NR]);

        g.execute(&context);
        g.execute(&context);
        g.execute(&context);
        g.execute(&context);
        g.execute(&context);
        assert_eq!(g.stack.len(), 1);
        assert_eq!(g.stack[0], 1);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_gene_execute_beyond_end() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[3, 4, ADD_NR]);

        g.execute(&context); // 3
        g.execute(&context); // 4
        g.execute(&context); // 7
        g.execute(&context); // 7 3
        g.execute(&context); // 7 3 4
        g.execute(&context); // 7 7
        assert_eq!(g.stack.len(), 2);
        assert_eq!(g.stack[0], 7);
        assert_eq!(g.stack[0], 7);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_gene_execute_nearby() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[3, 4, ADD_NR + 1, 6, SUB_NR - 1]);

        g.execute(&context);
        g.execute(&context);
        g.execute(&context);
        g.execute(&context);
        g.execute(&context);
        assert_eq!(g.stack.len(), 1);
        assert_eq!(g.stack[0], 1);
        assert_eq!(g.failures, 0);
    }

    #[test]
    fn test_gene_execute_stack_underflow() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 1000,
        };

        let mut g = Gene::new(&[4, ADD_NR]);

        g.execute(&context);
        g.execute(&context);
        assert_eq!(g.stack.len(), 0);
        assert_eq!(g.failures, 1);
    }

    #[test]
    fn test_gene_execute_stack_overflow_numbers() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 4,
        };

        let mut g = Gene::new(&[1, 2, 3, 4, 5]);

        g.execute(&context); // 1
        g.execute(&context); // 1 2
        g.execute(&context); // 1 2 3
        g.execute(&context); // 1 2 3 4
        g.execute(&context); // 3 4 5
        assert_eq!(g.stack.len(), 3);
        assert_eq!(g.failures, 1);
        assert_eq!(g.stack[0], 3);
        assert_eq!(g.stack[1], 4);
        assert_eq!(g.stack[2], 5);
    }

    #[test]
    fn test_gene_execute_stack_overflow_instructions() {
        let context = ExecutionContext {
            instruction_lookup: &instruction_lookup(),
            max_stack_size: 4,
        };

        let mut g = Gene::new(&[1, DUP_NR, DUP_NR, DUP_NR, DUP_NR]);

        g.execute(&context); // 1
        g.execute(&context); // 1 1
        g.execute(&context); // 1 1 1
        g.execute(&context); // 1 1 1 1
        g.execute(&context); // 1 1 1 1 1
        assert_eq!(g.stack.len(), 3);
        assert_eq!(g.failures, 1);
        assert_eq!(g.stack[0], 1);
        assert_eq!(g.stack[1], 1);
        assert_eq!(g.stack[2], 1);
    }

    #[test]
    fn test_add_execute() {
        let mut s: Vec<u32> = vec![4, 3];
        let b = Instruction::Add.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [7]);
    }

    #[test]
    fn test_add_execute_overflow() {
        let mut s: Vec<u32> = vec![u32::max_value(), 1];
        let b = Instruction::Add.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s, []);
    }

    #[test]
    fn test_add_execute_stack_underflow_empty_stack() {
        let mut s: Vec<u32> = vec![];
        let b = Instruction::Add.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s, []);
    }

    #[test]
    fn test_add_execute_stack_underflow_too_little_on_stack() {
        let mut s: Vec<u32> = vec![4];
        let b = Instruction::Add.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s, []);
    }

    #[test]
    fn test_sub_execute() {
        let mut s: Vec<u32> = vec![4, 3];
        let b = Instruction::Sub.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [1]);
    }

    #[test]
    fn test_sub_execute_underflow() {
        let mut s: Vec<u32> = vec![4, 5];
        let b = Instruction::Sub.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s, []);
    }

    #[test]
    fn test_mul_execute() {
        let mut s: Vec<u32> = vec![4, 3];
        let b = Instruction::Mul.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [12]);
    }

    #[test]
    fn test_div_execute() {
        let mut s: Vec<u32> = vec![12, 3];
        let b = Instruction::Div.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [4]);
    }

    #[test]
    fn test_eq_execute() {
        let mut s: Vec<u32> = vec![12, 12];
        let b = Instruction::Eq.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [TRUE]);
    }

    #[test]
    fn test_eq_execute_not_equal() {
        let mut s: Vec<u32> = vec![12, 3];
        let b = Instruction::Eq.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [FALSE]);
    }

    #[test]
    fn test_ne_execute() {
        let mut s: Vec<u32> = vec![12, 12];
        let b = Instruction::Ne.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [FALSE]);
    }

    #[test]
    fn test_ne_execute_not_equal() {
        let mut s: Vec<u32> = vec![12, 3];
        let b = Instruction::Ne.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [TRUE]);
    }

    #[test]
    fn test_gt_execute_true() {
        let mut s: Vec<u32> = vec![12, 3];
        let b = Instruction::Gt.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [TRUE]);
    }

    #[test]
    fn test_gt_execute_false() {
        let mut s: Vec<u32> = vec![3, 12];
        let b = Instruction::Gt.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [FALSE]);
    }

    #[test]
    fn test_lt_execute_true() {
        let mut s: Vec<u32> = vec![3, 12];
        let b = Instruction::Lt.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [TRUE]);
    }

    #[test]
    fn test_and_execute_true() {
        let mut s: Vec<u32> = vec![3, 1];
        let b = Instruction::And.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [TRUE]);
    }

    #[test]
    fn test_and_execute_false() {
        let mut s: Vec<u32> = vec![3, 0];
        let b = Instruction::And.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [FALSE]);
    }

    #[test]
    fn test_and_execute_false_both() {
        let mut s: Vec<u32> = vec![0, 0];
        let b = Instruction::And.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [FALSE]);
    }

    #[test]
    fn test_or_execute_true_both() {
        let mut s: Vec<u32> = vec![3, 1];
        let b = Instruction::Or.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [TRUE]);
    }

    #[test]
    fn test_or_execute_true_one() {
        let mut s: Vec<u32> = vec![3, 0];
        let b = Instruction::Or.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [TRUE]);
    }

    #[test]
    fn test_or_execute_false_both() {
        let mut s: Vec<u32> = vec![0, 0];
        let b = Instruction::Or.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [FALSE]);
    }

    #[test]
    fn test_not_false_to_true() {
        let mut s: Vec<u32> = vec![FALSE];
        let b = Instruction::Not.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [TRUE]);
    }

    #[test]
    fn test_not_true_to_false() {
        let mut s: Vec<u32> = vec![TRUE];
        Instruction::Not.execute(&mut s);
        assert_eq!(s, [FALSE]);
    }

    #[test]
    fn test_not_any_non_0_to_false() {
        let mut s: Vec<u32> = vec![123];
        Instruction::Not.execute(&mut s);
        assert_eq!(s, [FALSE]);
    }

    #[test]
    fn test_dup_execute() {
        let mut s: Vec<u32> = vec![12, 3];
        let b = Instruction::Dup.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [12, 3, 3]);
    }

    #[test]
    fn test_dup_execute_stack_underflow() {
        let mut s: Vec<u32> = vec![];
        let b = Instruction::Dup.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s, []);
    }

    #[test]
    fn test_drop_execute() {
        let mut s: Vec<u32> = vec![12, 3];
        let b = Instruction::Drop.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [12]);
    }

    #[test]
    fn test_swap_execute() {
        let mut s: Vec<u32> = vec![12, 3];
        let b = Instruction::Swap.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [3, 12]);
    }

    #[test]
    fn test_swap_execute_underflow() {
        let mut s: Vec<u32> = vec![12];
        let b = Instruction::Swap.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s, []);
    }

    #[test]
    fn test_over_execute() {
        let mut s: Vec<u32> = vec![12, 3];
        let b = Instruction::Over.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [12, 3, 12]);
    }

    #[test]
    fn test_over_stack_underflow() {
        let mut s: Vec<u32> = vec![12];
        let b = Instruction::Over.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s, []);
    }

    #[test]
    fn test_over_empty_stack() {
        let mut s: Vec<u32> = vec![];
        let b = Instruction::Over.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s, []);
    }

    #[test]
    fn test_rot_execute() {
        let mut s: Vec<u32> = vec![1, 2, 3];
        let b = Instruction::Rot.execute(&mut s);
        assert!(b.is_some());
        assert_eq!(s, [2, 3, 1]);
    }

    #[test]
    fn test_rot_execute_underflow() {
        let mut s: Vec<u32> = vec![1, 2];
        let b = Instruction::Rot.execute(&mut s);
        assert!(b.is_none());
        assert_eq!(s, []);
    }
}
