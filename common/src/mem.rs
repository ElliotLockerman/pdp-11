
use bytemuck::cast_slice;

pub fn as_word_slice(input: &[u8]) -> &[u16] {
    cast_slice(input)
}

pub fn as_byte_slice(input: &[u16]) -> &[u8] {
    cast_slice(input)
}

