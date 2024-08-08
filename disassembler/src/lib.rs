
use common::mem::as_word_slice;
use common::decoder::decode;
use common::constants::WORD_SIZE;

pub struct Disassembled {
    pub addr: u16,
    pub repr: Vec<u16>,
    pub interp: Option<String>,
}

pub fn disassemble(bin: &[u8]) -> Vec<Disassembled> {
    assert!(bin.len() <= (u16::MAX as usize) + 1);
    let mut out = vec![];
    let mut addr: usize = 0;
    while addr < bin.len() {
        let upper = usize::min(addr + 3 * WORD_SIZE as usize, bin.len());
        let ins = decode(as_word_slice(&bin[addr..upper]));
        if let Some(ins) = ins {
            out.push(Disassembled{
                addr: addr as u16,
                repr: as_word_slice(&bin[addr..addr + ins.size() as usize]).into(),
                interp: Some(format!("{:?}", ins)),
            });
            addr += ins.size() as usize;
        } else {
            out.push(Disassembled{
                addr: addr as u16,
                repr: as_word_slice(&bin[addr..addr + WORD_SIZE as usize]).into(),
                interp: None,
            });
            addr += WORD_SIZE as usize;
        }
    }

    out
}


