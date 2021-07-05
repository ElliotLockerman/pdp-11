
use std::convert::TryInto;

use crate::common::asm::*;


// Args are src, dst
#[derive(Debug)]
pub enum Stmt {
    LabelDef(String),
    Bytes(Vec<u8>),
    Words(Vec<u16>),
    Ascii(Vec<u8>),

    Ins(Ins),
}

impl Stmt {
    // Size, in bytes, of assembled statement
    pub fn size(&self) -> u16 {
        match self {
            Stmt::LabelDef(_) => 0,
            Stmt::Bytes(v) => v.len().try_into().unwrap(),
            Stmt::Words(v) => (v.len() * 2).try_into().unwrap(),
            Stmt::Ascii(v) => v.len().try_into().unwrap(),
            Stmt::Ins(ins) => ins.size(),
        }
    }
}


