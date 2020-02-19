use crate::lookup::Coordinates;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Gene {
    pub id: u32,
    pub code: RefCell<Vec<u32>>,
}

impl Gene {
    pub fn new(id: u32, code: &[u32]) -> Gene {
        return Gene {
            id: id,
            code: RefCell::new(code.to_vec()),
        };
    }
}

impl Coordinates for Gene {
    fn coordinates(&self) -> u32 {
        self.code.borrow()[0] & 0xFFFFFF
    }
}

impl Coordinates for Rc<Gene> {
    fn coordinates(&self) -> u32 {
        self.code.borrow()[0] & 0xFFFFFF
    }
}
