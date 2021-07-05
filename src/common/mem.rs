
pub unsafe fn as_word_slice(input: &[u8]) -> &[u16] {
    assert_eq!(input.len() & 0x1, 0);
    std::slice::from_raw_parts(input.as_ptr() as *const u16, input.len() / 2)
}

pub unsafe fn as_byte_slice(input: &[u16]) -> &[u8] {
    std::slice::from_raw_parts(input.as_ptr() as *const u8, input.len() * 2)
}

