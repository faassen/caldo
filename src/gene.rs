use crate::lookup::Coordinates;
use std::rc::Rc;

pub struct Gene<'a> {
    // id: u32, // unique per cell
    pub code: &'a [u32],
}

impl<'a> Gene<'a> {
    pub fn new(code: &[u32]) -> Gene {
        return Gene { code: code };
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
