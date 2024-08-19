
use std::io::Write;

use bytemuck::cast_slice;

pub fn as_word_slice(input: &[u8]) -> &[u16] {
    cast_slice(input)
}

pub fn as_byte_slice(input: &[u16]) -> &[u8] {
    cast_slice(input)
}

pub fn write_u16(out: &mut impl Write, word: u16) {
    let lower = word as u8;
    let upper = (word >> u8::BITS) as u8;
    out.write_all(&[lower, upper]).unwrap();
}

