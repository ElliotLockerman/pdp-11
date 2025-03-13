use std::io::{Read, Write};

use bytemuck::cast_slice;

pub fn as_word_slice(input: &[u8]) -> &[u16] {
    cast_slice(input)
}

pub fn as_byte_slice(input: &[u16]) -> &[u8] {
    cast_slice(input)
}

////////////////////////////////////////////////////////////////////////////////

pub trait ReadU16 {
    fn read_u16(&mut self) -> u16;
}

impl<T: Read> ReadU16 for T {
    fn read_u16(&mut self) -> u16 {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf).unwrap();
        let lower = buf[0] as u16;
        let upper = buf[1] as u16;
        lower | (upper << u8::BITS)
    }
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

////////////////////////////////////////////////////////////////////////////////

pub trait IsEven {
    fn is_even(self) -> bool;
}

impl IsEven for u16 {
    fn is_even(self) -> bool {
        self & 0x1 != 1
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait ToU16 {
    fn to_u16(self) -> u16;
}

impl ToU16 for usize {
    fn to_u16(self) -> u16 {
        assert!(self <= u16::MAX as usize);
        self as u16
    }
}
