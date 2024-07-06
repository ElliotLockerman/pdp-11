
use std::convert::TryInto;

use crate::common::asm::*;


// Args are src, dst
#[derive(Debug)]
pub enum Cmd {
    Bytes(Vec<u8>),
    Words(Vec<u16>),
    Ascii(Vec<u8>),

    Ins(Ins),
}


#[derive(Debug)]
pub struct Stmt {
    pub label_def: Option<String>,
    pub cmd: Option<Cmd>,
}
impl Stmt {
    pub fn new(label_def: Option<String>, cmd: Option<Cmd>) -> Stmt {
        assert!(label_def.is_some() || cmd.is_some());
        Stmt{label_def, cmd}
    }

    // Size, in bytes, of assembled statement
    pub fn size(&self) -> u16 {
        let Some(cmd) = &self.cmd else {
            return 0;
        };

        match cmd {
            Cmd::Bytes(v) => v.len().try_into().unwrap(),
            Cmd::Words(v) => (v.len() * 2).try_into().unwrap(),
            Cmd::Ascii(v) => v.len().try_into().unwrap(),
            Cmd::Ins(ins) => ins.size(),
        }
    }
}


