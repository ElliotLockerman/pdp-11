 
use std::collections::HashMap;
use std::convert::TryInto;

use crate::ir::*;
use crate::grammar::StmtParser;
use common::asm::*;
use common::constants::WORD_SIZE;

use log::trace;
use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////

pub struct Program {
    pub text: Vec<u8>,
    pub symbols: HashMap<String, SymbolValue>,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    Regular,
    Label,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Abs,
    Relc,
    Ext,
}

impl Mode {
    // Returns None if illegal.
    pub fn op_mode(lhs: Mode, op: Op, rhs: Mode) -> Option<Mode> {
        use Mode::*;
        match (lhs, op, rhs) {
            (Relc, Op::Sub, Relc) => Some(Abs), // Iff same section.
            (Abs, _, Abs) => Some(Abs),
            (Relc, _, Abs) => Some(Relc),
            (Abs, Op::Add, Relc) => Some(Relc),
            (Ext, _, Abs) => Some(Ext),
            (Abs, Op::Add, Ext) => Some(Ext),
            _ => None,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sect {
    Asect,
    CsectUn,
    CsectSym(String),
}

impl Sect {
    fn mode(&self) -> Mode {
        match self {
            Sect::Asect => Mode::Abs,
            _ => Mode::Relc,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub struct Value {
    pub val: u16,
    pub mode: Mode,
}

impl Value {
    fn new(val: u16, mode: Mode) -> Self {
        Value{val, mode}
    }
}

impl std::ops::Add<Value> for Value {
    type Output = Result<Value, EvalError>;
    fn add(self, rhs: Value) -> Self::Output {
        let mode = Mode::op_mode(self.mode, Op::Add, rhs.mode)
            .ok_or(EvalError::IllegalExpr(self, Op::Add, rhs))?;
        Ok(Value{val: self.val.wrapping_add(rhs.val), mode})
    }
}

impl std::ops::Sub<Value> for Value {
    type Output = Result<Value, EvalError>;
    fn sub(self, rhs: Value) -> Self::Output {
        let mode = Mode::op_mode(self.mode, Op::Sub, rhs.mode)
            .ok_or(EvalError::IllegalExpr(self, Op::Sub, rhs))?;
        Ok(Value{val: self.val.wrapping_sub(rhs.val), mode})
    }
}

impl std::ops::BitAnd<Value> for Value {
    type Output = Result<Value, EvalError>;
    fn bitand(self, rhs: Value) -> Self::Output {
        let mode = Mode::op_mode(self.mode, Op::And, rhs.mode)
            .ok_or(EvalError::IllegalExpr(self, Op::And, rhs))?;
        Ok(Value{val: self.val & rhs.val, mode})
    }
}

impl std::ops::BitOr<Value> for Value {
    type Output = Result<Value, EvalError>;
    fn bitor(self, rhs: Value) -> Self::Output {
        let mode = Mode::op_mode(self.mode, Op::Or, rhs.mode)
            .ok_or(EvalError::IllegalExpr(self, Op::Or, rhs))?;
        Ok(Value{val: self.val | rhs.val, mode})
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("Unable to resolve symbol")]
    SymbolUnresolved,

    #[error("Illegal Expr: {0:?} {} {2:?}", .1.to_char())]
    IllegalExpr(Value, Op, Value),
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct SymbolValue {
    pub val: u16,
    pub mode: Mode,
    pub typ: SymbolType,
    pub sect: Option<Sect>,
    pub line: usize,
}

impl SymbolValue {
    fn new(typ: SymbolType, val: Value, line: usize) -> SymbolValue {
        Self{
            typ,
            val: val.val,
            mode: val.mode,
            sect: None,
            line
        }
    }
}


////////////////////////////////////////////////////////////////////////////////

struct Assembler {
    buf: Vec<u8>,
    symbols: HashMap<String, SymbolValue>,
    sect: Sect,
}


impl Assembler {

    fn new() -> Assembler {
        Assembler{
            buf: Vec::new(),
            symbols: HashMap::new(),
            sect: Sect::Asect, // TODO: default should be relocatable.
        }
    }

    fn eval_atom(&self, atom: &Atom, loc: u16) -> Result<Value, EvalError> {
        match atom {
            Atom::Loc => Ok(Value::new(loc, self.sect.mode())),
            Atom::Val(val) => Ok(Value::new(*val, Mode::Abs)),
            Atom::SymbolRef(symbol) =>
                self.symbols.get(symbol).cloned()
                    .map(|x| Value::new(x.val, x.mode))
                    .ok_or(EvalError::SymbolUnresolved),
        }
    }

    fn eval_expr(&self, expr: &Expr, loc: u16) -> Result<Value, EvalError> {
        match expr {
            Expr::Atom(atom) => self.eval_atom(atom, loc),
            Expr::Op(lhs, op, rhs) => {
                let lhs = self.eval_expr(lhs, loc)?;
                let rhs = self.eval_atom(rhs, loc)?;

                match op {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::And => lhs & rhs,
                    Op::Or => lhs | rhs,
                }
            },
        }
    }

    fn resolve_operand(&self, arg: &mut Operand, curr_addr: &mut u16, loc: u16) {
        let val = match &arg.extra {
            Extra::None => return,
            Extra::Imm(expr) => {
                let val = self.eval_expr(expr, loc);
                if let (Expr::Atom(Atom::SymbolRef(symbol)), Ok(val)) = (expr, &val) {
                    trace!("Resolving symbol \"{symbol}\" (imm) to val 0o{:o}, curr_addr: 0o{curr_addr:o}", val.val);
                }
                val
            },
            Extra::Rel(expr) => {
                self.eval_expr(expr, loc).map(|val| {
                    assert!(val.mode == self.sect.mode());

                    let off = (val.val as i32 - *curr_addr as i32 - 2) as u16;

                    if let Expr::Atom(Atom::SymbolRef(symbol)) = expr {
                        trace!("Resolving symbol \"{symbol}\" (rel) to offset 0o{off:o}, curr_addr: 0o{curr_addr:o}, final: 0o{:o}", val.val);
                    }
                    Value::new(off, self.sect.mode())
                })
            }
        };

        match val {
            Ok(val) => arg.extra = Extra::Imm(Expr::Atom(Atom::Val(val.val))),
            Err(EvalError::SymbolUnresolved) => (),
            Err(e) => panic!("{e}"),
        }
        *curr_addr += WORD_SIZE;
    }

    fn resolve_target(&self, target: &mut Target, curr_addr: u16) {
        let offset = match target {
            Target::Offset(x) => *x,
            Target::Label(ref label) => {
                if let Some(sym) = self.symbols.get(label) {
                    let dst = sym.val;
                    let addr = curr_addr as i32;
                    TryInto::<i8>::try_into((dst as i32 - addr - 2)/2).unwrap() as u8
                } else {
                    return;
                }
            },
        };
        *target = Target::Offset(offset);
    }

    const MAX_ITER: i32 = 2;

    fn resolve_and_eval(&mut self, prog: &mut [Stmt]) {
        for _ in 1..=Self::MAX_ITER {
            let mut addr: u16 = 0;
            for (l, stmt) in prog.iter_mut().enumerate() {
                let line = l + 1;

                if let Some(label) = &stmt.label_def {
                    let sym = SymbolValue::new(SymbolType::Label, Value::new(addr, self.sect.mode()), line);
                    let existing = self.symbols.insert(label.clone(), sym); 
                    if let Some(existing) = existing {
                        if existing.line != line {
                            panic!("Label '{label}' on line {line} conflicts with previous definition on line {}", existing.line);
                        }
                    }
                }

                if stmt.cmd.is_none() {
                    continue;
                }

                let loc = addr;
                match stmt.cmd.as_mut().unwrap() {
                    Cmd::SymbolDef(symbol, expr) => {
                        if let Ok(val) = self.eval_expr(expr, loc) {
                            let sym = SymbolValue::new(SymbolType::Regular, val, line);
                            let existing = self.symbols.insert(symbol.clone(), sym.clone());
                            if let Some(existing) = existing {
                                if existing.typ == SymbolType::Label {
                                    panic!("Symbol '{symbol}' on line {line} conflicts with label on line {}", existing.line);
                                }
                                // Regular symbols are allowed to overwrite each other.
                            }
                        }
                       
                        
                    },
                    Cmd::Ins(ins) => {
                        match ins {
                            Ins::Branch(ins) => self.resolve_target(&mut ins.target, addr),
                            Ins::DoubleOperand(ins) => {
                                self.resolve_operand(&mut ins.src, &mut addr, loc);
                                self.resolve_operand(&mut ins.dst, &mut addr, loc);
                            },
                            Ins::Jmp(ins) => self.resolve_operand(&mut ins.dst, &mut addr, loc),
                            Ins::Jsr(ins) => self.resolve_operand(&mut ins.dst, &mut addr, loc),
                            Ins::SingleOperand(ins) => self.resolve_operand(&mut ins.dst, &mut addr, loc),
                            Ins::Eis(ins) => self.resolve_operand(&mut ins.operand, &mut addr, loc),
                            Ins::Trap(ins) => {
                                if let Ok(val) = self.eval_expr(&ins.data, loc) {
                                    assert_eq!(val.val & !0xff, 0);
                                    ins.data = Expr::Atom(Atom::Val(val.val)); 
                                }
                            },
                            _ => (),
                        }
                        addr += WORD_SIZE;
                    }
                    Cmd::Bytes(exprs) => {
                        for e in exprs {
                            if let Ok(val) = self.eval_expr(e, addr) {
                                *e = Expr::Atom(Atom::Val(val.val));
                            }
                            addr += 1;
                        }
                    }
                    Cmd::Words(exprs) => {
                        for e in exprs {
                            if let Ok(val) = self.eval_expr(e, addr) {
                                *e = Expr::Atom(Atom::Val(val.val));
                            }
                            addr += WORD_SIZE;
                        }
                    },
                    Cmd::LocDef(expr) => {
                        if let Ok(val) = self.eval_expr(expr, addr) {
                            assert!(val.val >= addr);
                            addr = val.val;
                            *expr = Expr::Atom(Atom::Val(addr))
                        }
                    },
                    Cmd::Even => {
                        if addr & 0x1 != 0 {
                            addr += 1;
                        }
                    },
                    _ => { addr += stmt.size().unwrap(); },
                }
            }
        }
    }

    fn check_resolved(&self, prog: &[Stmt]) {
        for (l, stmt) in prog.iter().enumerate() {
            if let Err(e) = stmt.check_resolved() {
                panic!("Line {}: Unable to resolve '{}'", l + 1, e.0);
            }
        }
    }

    fn assemble(mut self, prog: &str) -> Program {

        let lines = prog.split('\n');
        let parser = StmtParser::new();

        let prog: Vec<_> = lines
            .zip(1..)
            .map(|(x, i)| parser.parse(x).map_err(|e| {
                eprintln!("Error line {i}: {e}"); e
            })).collect();

        let mut prog: Vec<_> = prog.into_iter()
            .map(|x| x.unwrap_or_else(|_| panic!("Exiting due to previous errors")))
            .filter(|x| !x.is_empty())
            .collect();

        self.resolve_and_eval(&mut prog);
        self.check_resolved(&prog);

        for stmt in prog {
            stmt.emit(&mut self.buf);
        }
        Program{text: self.buf, symbols: self.symbols}
    }
}


pub fn assemble(prog: &str) -> Program {
    Assembler::new().assemble(prog)
}

#[cfg(test)]
mod tests {
    use super::assemble;
    fn to_u16(arr: &Vec<u8>) -> Vec<u16> {
        assert_eq!(arr.len() % 2, 0);
        let mut out = Vec::new();
        for chunk in arr.chunks(2) {
            out.push((chunk[0] as u16) | (chunk[1] as u16) << 8);
        }
        out
    }

    #[test]
    fn halt() {
        let prog = "halt";
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0);
    }

    #[test]
    fn mov_reg_reg() {
        let prog = "mov r0, r1";
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o10001);
    }

    #[test]
    fn mov_mem_mem() {
        let prog = "mov (r0)+, -(r1)";
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o12041);
    }

    #[test]
    fn branch() {
        let prog = r#"
            label:
                br label"#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o000777);
    }

    #[test]
    fn numbers() {
        let prog = r#"
            .word 0
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o0);

        let prog = r#"
            .word 7
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o7);

        let prog = r#"
            .word 17
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o17);

        let prog = r#"
            .word 0.
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0);

        let prog = r#"
            .word 7.
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 7);

        let prog = r#"
            .word 17.
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 17);
    }

    #[test]
    fn negative_numbers() {
        let prog = r#"
            .word -0
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o0);

        let prog = r#"
            .word -7
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], -0o7i16 as u16);

        let prog = r#"
            .word 17
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o17);

        let prog = r#"
            .word 0.
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0);

        let prog = r#"
            .word 7.
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 7);

        let prog = r#"
            .word 17.
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 17);
    }

    #[test]
    #[should_panic]
    fn byte_size() {
        let prog = r#"
            .byte 400
        "#;
        assemble(prog).text;
    }

    #[test]
    #[should_panic]
    fn word_size() {
        let prog = r#"
            .word 200000
        "#;
        assemble(prog).text;
    }

    #[test]
    fn char_literal() {
        let prog = r#"
            .byte 'A
        "#;
        let bin = assemble(prog).text;
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0x41);
    }

    #[test]
    fn symbol() {
        let prog = r#"
            SYM = 37
            mov #SYM, r0
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[1], 0o37);
    }

    #[test]
    fn forward_symbol() {
        let prog = r#"
            a = b
            b = 37
            mov #a, r0
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[1], 0o37);
    }

    #[test]
    #[should_panic]
    fn not_too_forward() {
        let prog = r#"
            a = b
            b = c
            c = 37
            mov #a, r0
        "#;
        assemble(prog);
    }

    #[test]
    #[should_panic]
    fn never_defined() {
        let prog = r#"
            a = b
            mov #a, r0
        "#;
        assemble(prog);
    }

    #[test]
    fn symbol_byte() {
        let prog = r#"
            a = 37
            .byte a
        "#;
        let bin = assemble(prog).text;
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o37);
    }

    #[test]
    fn symbol_word() {
        let prog = r#"
            a = 777
            .word a
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o777);
    }

    #[test]
    fn expr_word() {
        let prog = r#"
            .word 2 + 1
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o3);

        let prog = r#"
            .word 1 + 1 ! 2
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o2);

        let prog = r#"
            .word 1 ! 2 + 1
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o4);
    }

    #[test]
    fn expr_byte() {
        let prog = r#"
            .byte 2 + 1
        "#;
        let bin = &assemble(prog).text;
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o3);

        let prog = r#"
            .byte 1 + 1 ! 2
        "#;
        let bin = &assemble(prog).text;
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o2);

        let prog = r#"
            .byte 1 ! 2 + 1
        "#;
        let bin = &assemble(prog).text;
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o4);
    }


    #[test]
    #[should_panic] // The manual says to truncate, I've chosen not to.
    fn expr_byte_overflow() {
        let prog = r#"
            .byte 377 + 1
        "#;
        let bin = &assemble(prog).text;
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o0);
    }

    #[test]
    fn expr_index() {
        let prog = r#"
            FIELD_A = 2 + 2
            mov FIELD_A(r0), r1
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[1], 0o4);

        let prog = r#"
            FIELD_A = 4
            mov FIELD_A + 2(r0), r1
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[1], 0o6);
    }

    #[test]
    fn period_expr() {
        let prog = r#"
            .word .
        "#;
        let bin = &assemble(prog).text;
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[0], 0o0);

        let prog = r#"
            .word ., .
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[0], 0o0);
        assert_eq!(bin[1], 0o2);

        let prog = r#"
            clr r0
            mov #., r0
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[2], 0o2);

        let prog = r#"
            .word 0, 0
            loc = .
            .word loc
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[2], 0o4);

        let prog = r#"
            .word 0, 0
            .word loc
            loc = .
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[2], 0o6);
    }


    #[test]
    fn period_assign() {
        let prog = r#"
            . = 12
        "#;
        let bin = &assemble(prog).text;
        assert_eq!(bin.len(), 10);

        let prog = r#"
            . = 2
            mov #., r0
        "#;
        let bin = to_u16(&assemble(prog).text);
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[2], 2);
    }

    #[test]
    fn even() {
        let prog = r#"
            .byte 0
        "#;
        let bin = assemble(prog).text;
        assert_eq!(bin.len(), 1);

        let prog = r#"
            .byte 0
            .even
        "#;
        let bin = assemble(prog).text;
        assert_eq!(bin.len(), 2);

        let prog = r#"
            . = 11
        "#;
        let bin = assemble(prog).text;
        assert_eq!(bin.len(), 9);

        let prog = r#"
            . = 11
            .even
        "#;
        let bin = assemble(prog).text;
        assert_eq!(bin.len(), 10);
    }

    #[test]
    fn eis() {
        let bin = to_u16(&assemble(r#"mul r1, r0"#).text);
        assert_eq!(bin, [0o070001]);

        let bin = to_u16(&assemble(r#"div @(r2)+, r4"#).text);
        assert_eq!(bin, [0o071432]);

        let bin = to_u16(&assemble(r#"ash #23, r5"#).text);
        assert_eq!(bin, [0o072527, 0o23]);

        let bin = to_u16(&assemble(r#"label: ashc label, r5"#).text);
        assert_eq!(bin, [0o073567, 0o177776]);

        let bin = to_u16(&assemble(r#"label: xor label, r5"#).text);
        assert_eq!(bin, [0o074567, 0o177776]);
    }

    #[test]
    #[should_panic]
    fn div_odd() {
        assemble(r#"div @(r2).text+, r5"#);
    }

    #[test]
    #[should_panic]
    fn label_redef_0() {
        assemble(r#"
        label:
        label:
        "#);
    }

    #[test]
    #[should_panic]
    fn label_redef_1() {
        assemble(r#"
        label:
        label = 1
        "#);
    }

    #[test]
    #[should_panic]
    fn label_redef_2() {
        assemble(r#"
        label = 1
        label:
        "#);
    }

    #[test]
    fn label_redef_3() {
        let prog = assemble(r#"
        label = 1
        label = 2
        "#);
        assert_eq!(prog.symbols.get("label").unwrap().val, 2);
    }

    #[test]
    fn mode_arith() {
        use super::Mode::{self, *};
        use super::Op::*;

        // From PAL-11 4-4
        assert_eq!(Mode::op_mode(Ext, Sub, Abs), Some(Ext));
        assert_eq!(Mode::op_mode(Abs, Add, Ext), Some(Ext));

        assert_eq!(Mode::op_mode(Relc, Add, Abs), Some(Relc));
        assert_eq!(Mode::op_mode(Abs, Add, Relc), Some(Relc));
        assert_eq!(Mode::op_mode(Abs, Add, Relc), Some(Relc));
        assert_eq!(Mode::op_mode(Relc, Sub, Abs), Some(Relc));
        assert_eq!(Mode::op_mode(Abs, Add, Relc), Some(Relc));

        assert_eq!(Mode::op_mode(Relc, Sub, Relc), Some(Abs));

        assert_eq!(Mode::op_mode(Ext, Add, Relc), None);
        assert_eq!(Mode::op_mode(Relc, Add, Relc), None);
        assert_eq!(Mode::op_mode(Abs, Sub, Relc), None);
        assert_eq!(Mode::op_mode(Relc, And, Relc), None);
    }
}






