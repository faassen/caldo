pub struct Gene<'a> {
    pub code: &'a [u32],
}

impl<'a> Gene<'a> {
    pub fn new(code: &[u32]) -> Gene {
        return Gene { code: code };
    }
}
