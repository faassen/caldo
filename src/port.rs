pub struct Port<'a> {
    pub input: Vec<u32>,
    pub output: Vec<u32>,
    pub expected_output: &'a [u32],
}

impl<'a> Port<'a> {
    pub fn new() -> Port<'a> {
        Port {
            input: vec![],
            output: vec![],
            expected_output: &[],
        }
    }

    pub fn reset(&mut self, expected_input: &[u32], expected_output: &'a [u32]) {
        // we reverse the order of the elements so that we
        // can pop them off in the right order
        let mut cloned_input = expected_input.to_vec();
        cloned_input.reverse();
        self.input = cloned_input;
        self.output = vec![];
        self.expected_output = expected_output;
    }

    pub fn read(&mut self) -> u32 {
        if self.input.len() == 0 {
            return 0;
        }
        self.input.pop().unwrap()
    }

    pub fn write(&mut self, value: u32) {
        self.output.push(value);
    }

    pub fn is_done(&self) -> bool {
        self.output.len() >= self.expected_output.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port() {
        let mut port = Port::new();
        port.reset(&[10, 20], &[30]);
        assert_eq!(port.read(), 10);
        assert_eq!(port.read(), 20);
        assert_eq!(port.read(), 0);
        assert_eq!(port.read(), 0);
        assert!(!port.is_done());
        port.write(40);
        assert!(port.is_done());
    }
}
