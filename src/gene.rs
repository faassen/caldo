use crate::lookup::Coordinates;
use std::rc::Rc;

pub struct Gene<'a> {
    pub id: u32,
    pub code: &'a [u32],
}

impl<'a> Gene<'a> {
    pub fn new(id: u32, code: &[u32]) -> Gene {
        return Gene { id: id, code: code };
    }
}

impl<'a> Coordinates for Gene<'a> {
    fn coordinates(&self) -> u32 {
        self.code[0] & 0xFFFFFF
    }
}

impl<'a> Coordinates for Rc<Gene<'a>> {
    fn coordinates(&self) -> u32 {
        self.code[0] & 0xFFFFFF
    }
}
