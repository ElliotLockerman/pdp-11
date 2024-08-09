
pub fn parse_int(s: &str, base: u32) -> u16 {
    let neg = s.starts_with('-');
    let offset = neg as usize;
    let mut val = u16::from_str_radix(&s[offset..], base).unwrap();
    if neg {
        val = (!val).wrapping_add(1);
    }
    val
}
