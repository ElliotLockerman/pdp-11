use common::asm::Ins;
use common::constants::{MAX_INS_WORDS, WORD_SIZE};

use std::fmt;

use bytemuck::cast_slice;

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

        if let Some(ins) = &self.ins {
            write!(f, "{}", ins.display_with_pc(self.addr))?;
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
        let ins = Ins::decode(cast_slice(&bin[addr..upper]));
        let size = ins.as_ref().map(|x| x.size()).unwrap_or(WORD_SIZE) as usize;
        out.push(Disassembled {
            addr: addr as u16,
            repr: cast_slice(&bin[addr..addr + size]).into(),
            ins,
        });
        addr += size;
    }

    out
}
