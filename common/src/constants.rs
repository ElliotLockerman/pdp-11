pub const WORD_SIZE: u16 = 2; // Bytes
pub const MAX_INS_WORDS: u16 = 3; // Maximum number of words in an instruction.

pub const INTERUPT_START: u16 = 0o0;
pub const INTERUPT_END: u16 = DATA_START; // Exclusive
pub const DATA_START: u16 = 0o400;
pub const DATA_END: u16 = MMIO_START; // Exclusive
pub const MMIO_START: u16 = 0o160000;
pub const MEM_HIGH: u16 = 0o177777;
pub const MEM_END: u32 = 0o1000000; // Exclusive, note type
