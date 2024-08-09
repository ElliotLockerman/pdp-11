
use std::convert::TryInto;

use common::asm::*;


// Args are src, dst
#[derive(Debug)]
pub enum Cmd {
    Bytes(Vec<Expr>),
    Words(Vec<Expr>),
    Ascii(Vec<u8>),
    Even,

    Ins(Ins),
    SymbolDef(String, Expr),
    LocDef(Expr),
}


#[derive(Debug)]
pub struct Stmt {
    pub label_def: Option<String>,
    pub cmd: Option<Cmd>,
}
impl Stmt {
    pub fn new(label_def: Option<String>, cmd: Option<Cmd>) -> Stmt {
        Stmt{label_def, cmd}
    }

    pub fn is_empty(&self) -> bool {
        self.label_def.is_none() && self.cmd.is_none()
    }

    // Size, in bytes, of assembled statement, if known.
    pub fn size(&self) -> Option<u16> {
        let Some(cmd) = &self.cmd else {
            return Some(0);
        };

        let val = match cmd {
            Cmd::Bytes(v) => v.len().try_into().unwrap(),
            Cmd::Words(v) => (v.len() * 2).try_into().unwrap(),
            Cmd::Ascii(v) => v.len().try_into().unwrap(),
            Cmd::Ins(ins) => ins.size(),
            Cmd::SymbolDef(_, _) => 0,

            // Must be handled manually!
            Cmd::LocDef(_) | Cmd::Even => return None,
        };
        Some(val)
    }
}


