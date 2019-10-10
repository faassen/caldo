// Instructions that affect the stack only

const TRUE: u32 = 0xFFFFFFFF;
const FALSE: u32 = 0;

pub trait Stack<T> {
    fn pop2(&mut self) -> Option<(T, T)>;
}

impl<T> Stack<T> for Vec<T> {
    fn pop2(&mut self) -> Option<(T, T)> {
        return self
            .pop()
            .and_then(|first| self.pop().and_then(|second| Some((second, first))));
    }
}

pub trait OpStack<T, F>
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

impl Instruction {
    pub fn execute(&self, stack: &mut Vec<u32>) -> Option<()> {
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

fn bool_to_nr(b: bool) -> u32 {
    if b {
        TRUE
    } else {
        FALSE
    }
}

pub fn nr_to_bool(nr: u32) -> bool {
    nr != 0
}

#[cfg(test)]
mod tests {
    use super::*;

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
