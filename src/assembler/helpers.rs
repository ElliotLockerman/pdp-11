
fn get_offset(s: &str) -> usize {
    let mut offset = 0;
    if s.chars().nth(offset) == Some('0') && s.chars().nth(offset+1) == Some('x') {
        offset += 2;
    }
    offset
}

pub fn parse_int(s: &str, base: u32) -> u16 {
    let offset = get_offset(s);
    u16::from_str_radix(&s[offset..], base).unwrap()
}
