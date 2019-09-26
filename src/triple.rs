struct Triple {
    r: u8,
    g: u8,
    b: u8,
    instruction: bool,
}

impl Triple {
    fn from_int(i: u32) -> Triple {
        let fields = i >> 24 & 0xff;
        return Triple {
            r: (i >> 16 & 0xff) as u8,
            g: (i >> 8 & 0xff) as u8,
            b: (i & 0xff) as u8,
            instruction: (fields & 0x01) != 0,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_int_smaller() {
        let t = Triple::from_int(0x010203);

        assert_eq!(t.r, 1);
        assert_eq!(t.g, 2);
        assert_eq!(t.b, 3);
        assert_eq!(t.instruction, false)
    }

    #[test]
    fn test_from_int_bigger() {
        let t = Triple::from_int(0xFACBDE);

        assert_eq!(t.r, 0xFA);
        assert_eq!(t.g, 0xCB);
        assert_eq!(t.b, 0xDE);
        assert_eq!(t.instruction, false)
    }

    #[test]
    fn test_from_int_instruction_bit() {
        let t = Triple::from_int(0x01010203);

        assert_eq!(t.r, 1);
        assert_eq!(t.g, 2);
        assert_eq!(t.b, 3);
        assert_eq!(t.instruction, true)
    }

}
