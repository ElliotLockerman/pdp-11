
use common::mem::as_word_slice;
use common::decoder::decode;
use common::constants::{WORD_SIZE, MAX_INS_WORDS};
use common::asm::Ins;

use std::fmt;


fn write_oct_words(f: &mut fmt::Formatter, vals: &[u16]) -> fmt::Result {
    for i in 0..MAX_INS_WORDS {
        if (i as usize) < vals.len() {
            write!(f, "{:#08o}", vals[i as usize])?;
        } else {
            write!(f, "        ")?;
        }

        if i != MAX_INS_WORDS {
            write!(f, " ")?;
        }
    }
    Ok(())
}



pub struct Disassembled {
    pub addr: u16,
    pub repr: Vec<u16>,
    pub ins: Option<Ins>,
}


impl fmt::Display for Disassembled {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#08o}", self.addr)?;
        write!(f, "\t")?;
        write_oct_words(f, &self.repr)?;
        write!(f, "\t")?;

        match &self.ins {
            Some(Ins::Branch(br)) => br.display_with_pc(f, self.addr)?,
            Some(ins) => write!(f, "{}", ins)?,
            None => (),
        }
        Ok(())
    }
}


pub fn disassemble(bin: &[u8]) -> Vec<Disassembled> {
    assert!(bin.len() <= (u16::MAX as usize) + 1);
    let mut out = vec![];
    let mut addr: usize = 0;
    while addr < bin.len() {
        let upper = usize::min(addr + 3 * WORD_SIZE as usize, bin.len());
        let ins = decode(as_word_slice(&bin[addr..upper]));
        let size = ins.as_ref().map(|x| x.size()).unwrap_or(WORD_SIZE) as usize;
        out.push(Disassembled{
            addr: addr as u16,
            repr: as_word_slice(&bin[addr..addr + size]).into(),
            ins,
        });
        addr += size;
    }

    out
}


