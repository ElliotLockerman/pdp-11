use std::io::Write;

use bytemuck::cast_slice;

pub fn as_word_slice(input: &[u8]) -> &[u16] {
    cast_slice(input)
}

pub fn as_byte_slice(input: &[u16]) -> &[u8] {
    cast_slice(input)
}

////////////////////////////////////////////////////////////////////////////////

pub trait WriteU16 {
    fn write_u16(&mut self, val: u16);
}

impl<T: Write> WriteU16 for T {
    fn write_u16(&mut self, val: u16) {
        let lower = val as u8;
        let upper = (val >> u8::BITS) as u8;
        self.write_all(&[lower, upper]).unwrap();
    }
}
