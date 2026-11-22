//! LEB128, the variable-length integer encoding of the WebAssembly binary
//! format.

pub fn unsigned(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

pub fn signed(out: &mut Vec<u8>, mut value: i64) {
    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        let done = (value == 0 && byte & 0x40 == 0) || (value == -1 && byte & 0x40 != 0);
        if done {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}

/// A cursor over bytes being decoded.
pub struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Reader { bytes, pos: 0 }
    }

    pub fn at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    pub fn byte(&mut self) -> Option<u8> {
        let byte = *self.bytes.get(self.pos)?;
        self.pos += 1;
        Some(byte)
    }

    pub fn take(&mut self, count: usize) -> Option<&'a [u8]> {
        let slice = self.bytes.get(self.pos..self.pos.checked_add(count)?)?;
        self.pos += count;
        Some(slice)
    }

    /// Everything not yet read.
    pub fn rest(&mut self) -> &'a [u8] {
        let slice = &self.bytes[self.pos..];
        self.pos = self.bytes.len();
        slice
    }

    pub fn unsigned(&mut self) -> Option<u32> {
        let mut value: u64 = 0;
        for shift in (0..35).step_by(7) {
            let byte = self.byte()?;
            value |= u64::from(byte & 0x7F) << shift;
            if byte & 0x80 == 0 {
                return u32::try_from(value).ok();
            }
        }
        None
    }

    /// A length-prefixed UTF-8 name.
    pub fn name(&mut self) -> Option<&'a str> {
        let len = self.unsigned()? as usize;
        std::str::from_utf8(self.take(len)?).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(value: u64) -> Vec<u8> {
        let mut out = Vec::new();
        unsigned(&mut out, value);
        out
    }

    fn s(value: i64) -> Vec<u8> {
        let mut out = Vec::new();
        signed(&mut out, value);
        out
    }

    #[test]
    fn encodes_as_the_specification_says() {
        assert_eq!(u(0), [0x00]);
        assert_eq!(u(127), [0x7F]);
        assert_eq!(u(128), [0x80, 0x01]);
        assert_eq!(u(624485), [0xE5, 0x8E, 0x26]);
        assert_eq!(s(0), [0x00]);
        assert_eq!(s(-1), [0x7F]);
        assert_eq!(s(63), [0x3F]);
        assert_eq!(s(64), [0xC0, 0x00]);
        assert_eq!(s(-64), [0x40]);
        assert_eq!(s(-123456), [0xC0, 0xBB, 0x78]);
    }

    #[test]
    fn reads_back_what_was_written() {
        for value in [0u64, 1, 127, 128, 300, 65_535, u64::from(u32::MAX)] {
            assert_eq!(Reader::new(&u(value)).unsigned(), Some(value as u32));
        }
        assert_eq!(Reader::new(&[0x80]).unsigned(), None);
    }
}
