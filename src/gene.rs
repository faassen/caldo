use slotmap::new_key_type;

new_key_type! { pub struct GeneKey; }

pub struct Gene {
    pub id: u32,
    pub code: Vec<u32>,
}

impl Gene {
    pub fn new(id: u32, code: &[u32]) -> Gene {
        return Gene {
            id: id,
            code: code.to_vec(),
        };
    }

    pub fn coordinates(&self) -> u32 {
        self.code[0] & 0xFFFFFF
    }
}
