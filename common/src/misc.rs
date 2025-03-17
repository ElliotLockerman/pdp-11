use std::io::{Read, Write};

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

pub trait IsEven: Copy {
    #[allow(clippy::wrong_self_convention)]
    fn is_even(self) -> bool;
}

impl IsEven for u16 {
    fn is_even(self) -> bool {
        self & 0x1 != 1
    }
}

////////////////////////////////////////////////////////////////////////////////

// A panicking version.
pub trait ToU16P {
    fn to_u16p(self) -> u16;
}

impl ToU16P for usize {
    fn to_u16p(self) -> u16 {
        assert!(self <= u16::MAX as Self);
        self as u16
    }
}

impl ToU16P for u32 {
    fn to_u16p(self) -> u16 {
        assert!(self <= u16::MAX as Self);
        self as u16
    }
}
