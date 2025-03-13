use std::convert::TryInto;

use common::asm::*;
use common::misc::WriteU16;

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
        Stmt { label_def, cmd }
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

    pub fn emit(&self, buf: &mut Vec<u8>) {
        if let Some(cmd) = &self.cmd {
            match cmd {
                Cmd::Bytes(exprs) => {
                    for e in exprs {
                        let val = TryInto::<u8>::try_into(e.unwrap_val()).unwrap();
                        buf.push(val);
                    }
                }
                Cmd::Words(exprs) => {
                    for e in exprs {
                        buf.write_u16(e.unwrap_val());
                    }
                }
                Cmd::Ascii(a) => buf.extend(a),
                Cmd::Ins(ins) => ins.emit(buf),
                Cmd::SymbolDef(_, _) => (),
                Cmd::LocDef(addr) => {
                    let addr = addr.unwrap_val();
                    assert!(addr as usize >= buf.len());
                    buf.resize(addr as usize, 0);
                }
                Cmd::Even => {
                    if buf.len() & 0x1 == 1 {
                        buf.push(0);
                    }
                }
            }
        }
    }

    pub fn check_resolved(&self) -> Result<(), ResolvedError> {
        if self.cmd.is_none() {
            return Ok(());
        }
        match self.cmd.as_ref().unwrap() {
            Cmd::Ins(ins) => {
                ins.check_resolved()?;
            }
            Cmd::Bytes(exprs) => {
                for e in exprs {
                    e.check_resolved()?;
                }
            }
            Cmd::Words(exprs) => {
                for e in exprs {
                    e.check_resolved()?;
                }
            }
            Cmd::LocDef(expr) => {
                expr.check_resolved()?;
            }
            _ => (),
        }
        Ok(())
    }
}
