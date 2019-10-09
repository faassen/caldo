#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Mode {
    Number,
    Instruction,
    Call,
    Noop,
}

#[derive(Debug, Copy, Clone)]
pub struct Triplet {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub mode: Mode,
}

// XXX Use From and Into
impl Triplet {
    pub fn from_int(i: u32) -> Triplet {
        let fields = i >> 24 & 0xff;
        let mode = match fields & 0x01 {
            0 => Mode::Number,
            1 => Mode::Instruction,
            2 => Mode::Call,
            _ => Mode::Noop,
        };
        Triplet {
            r: (i >> 16 & 0xff) as u8,
            g: (i >> 8 & 0xff) as u8,
            b: (i & 0xff) as u8,
            mode: mode
        }
    }

    pub fn coordinates(&self) -> [f32; 3] {
        return [self.r as f32, self.g as f32, self.b as f32];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_int_smaller() {
        let t = Triplet::from_int(0x010203);

        assert_eq!(t.r, 1);
        assert_eq!(t.g, 2);
        assert_eq!(t.b, 3);
        assert_eq!(t.mode, Mode::Number)
    }

    #[test]
    fn test_from_int_bigger() {
        let t = Triplet::from_int(0xFACBDE);

        assert_eq!(t.r, 0xFA);
        assert_eq!(t.g, 0xCB);
        assert_eq!(t.b, 0xDE);
        assert_eq!(t.mode, Mode::Number)
    }

    #[test]
    fn test_from_int_instruction_bit() {
        let t = Triplet::from_int(0x01010203);

        assert_eq!(t.r, 1);
        assert_eq!(t.g, 2);
        assert_eq!(t.b, 3);
        assert_eq!(t.mode, Mode::Instruction)
    }

}
