use std::collections::HashMap;
use std::convert::TryInto;

use crate::grammar::StmtParser;
use crate::ir::*;
use crate::misc::{EvalError, Mode, Value};
use crate::tmp_f_tracker::TmpFTracker;
use aout::Aout;
use common::asm::*;
use common::constants::WORD_SIZE;
use common::misc::ToU16P;

use log::trace;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sect {
    Text,
    Data,
    Bss,
}

impl Sect {
    fn mode(&self) -> Mode {
        match self {
            Sect::Text => Mode::Text,
            Sect::Data => Mode::Data,
            Sect::Bss => Mode::Bss,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct SymbolValue {
    pub val: u16,
    pub mode: Mode,
    pub typ: SymbolType,
    pub line: usize,
}

impl SymbolValue {
    fn new(typ: SymbolType, val: Value, line: usize) -> SymbolValue {
        Self {
            typ,
            val: val.val,
            mode: val.mode,
            line,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

struct Assembler {
    buf: Vec<u8>,
    symbols: HashMap<String, SymbolValue>,
    tmp_symbols: HashMap<u16, SymbolValue>,
    sect: Sect,
    tmp_f_tracker: TmpFTracker,
    line: usize, // Current line.

    // Location counter, the period operator. Address of begining of current
    // instruction or when used in of current word.
    loc: u16,

    // Current address.
    addr: u16,
}

impl Assembler {
    fn new() -> Assembler {
        Assembler {
            buf: Vec::new(),
            symbols: HashMap::new(),
            tmp_symbols: HashMap::new(),
            sect: Sect::Text,
            tmp_f_tracker: TmpFTracker::new(),
            line: 0,
            loc: 0,
            addr: 0,
        }
    }

    fn eval_atom(&mut self, atom: &Atom) -> Result<Value, EvalError> {
        match atom {
            Atom::Loc => Ok(Value::new(self.loc, self.sect.mode())),
            Atom::Val(val) => Ok(Value::new(*val, Mode::Abs)),
            Atom::SymbolRef(symbol) => self
                .symbols
                .get(symbol)
                .map(|x| Value::new(x.val, x.mode))
                .ok_or(EvalError::SymbolUnresolved),

            Atom::TmpSymbolFRef(label) => match self.tmp_f_tracker.get(self.line, *label) {
                Some((addr, mode)) => Ok(Value::new(addr, mode)),
                None => {
                    self.tmp_f_tracker.need(self.line, *label);
                    Err(EvalError::SymbolUnresolved)
                }
            },
            Atom::TmpSymbolBRef(label) => self
                .tmp_symbols
                .get(label)
                .map(|x| Value::new(x.val, x.mode))
                .ok_or(EvalError::SymbolUnresolved),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, EvalError> {
        match expr {
            Expr::Atom(atom) => self.eval_atom(atom),
            Expr::Op(lhs, op, rhs) => {
                let lhs = self.eval_expr(lhs)?;
                let rhs = self.eval_atom(rhs)?;

                match op {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::And => lhs & rhs,
                    Op::Or => lhs | rhs,
                    _ => todo!(),
                }
            }
        }
    }

    fn eval_operand(&mut self, arg: &mut Operand) {
        if matches!(arg.extra, Extra::None) {
            return;
        }

        let val = match &arg.extra {
            Extra::None => unreachable!(),
            Extra::Imm(expr) => {
                let val = self.eval_expr(expr);
                if let (Expr::Atom(Atom::SymbolRef(symbol)), Ok(val)) = (expr, &val) {
                    trace!(
                        "Resolving symbol \"{symbol}\" (imm) to val 0o{:o}, addr: 0o{:o}",
                        val.val,
                        self.addr
                    );
                }
                val
            }
            Extra::Rel(expr) => {
                self.eval_expr(expr).map(|val| {
                    assert!(val.mode == Mode::Abs || val.mode == self.sect.mode());

                    let off = (val.val as i32 - self.addr as i32 - 2) as u16;

                    // TODO: doesn't cover temporary symbols.
                    if let Expr::Atom(Atom::SymbolRef(symbol)) = expr {
                        trace!("Resolving symbol \"{symbol}\" (rel) to offset 0o{off:o}, curr_addr: 0o{:o}, final: 0o{:o}", self.addr, val.val);
                    }
                    Value::new(off, self.sect.mode())
                })
            }
        };

        self.addr += WORD_SIZE;

        match val {
            Ok(val) => arg.extra = Extra::Imm(Expr::Atom(Atom::Val(val.val))),
            Err(EvalError::SymbolUnresolved) => (),
            Err(e) => panic!("{e}"),
        }
    }

    // Todo: return result instead of panicking for temporary labels.
    fn eval_target(&mut self, target: &mut Target) {
        let target_addr = match target {
            Target::Offset(x) => {
                *target = Target::Offset(*x);
                return;
            }
            &mut Target::Label(ref label) => match self.symbols.get(label) {
                Some(sym) => sym.val,
                None => return,
            },
            Target::TmpLabelF(label) => match self.tmp_f_tracker.get(self.line, *label) {
                Some((addr, _)) => addr,
                None => {
                    self.tmp_f_tracker.need(self.line, *label);
                    return;
                }
            },
            Target::TmpLabelB(label) => self.tmp_symbols[label].val,
        };

        let addr = self.loc as i32;
        *target = Target::Offset(
            TryInto::<i8>::try_into((target_addr as i32 - addr - 2) / 2).unwrap() as u8,
        );
    }

    fn eval_cmd(&mut self, cmd: &mut Cmd) {
        match cmd {
            Cmd::SymbolDef(symbol, expr) => {
                if let Ok(val) = self.eval_expr(expr) {
                    let sym = SymbolValue::new(SymbolType::Regular, val, self.line);
                    let existing = self.symbols.insert(symbol.clone(), sym.clone());
                    if let Some(existing) = existing {
                        if existing.typ == SymbolType::Label {
                            panic!(
                                "Symbol '{symbol}' on line {} conflicts with label on line {}",
                                self.line, existing.line
                            );
                        }
                        // Regular symbols are allowed to overwrite each other.
                    }
                }
            }
            Cmd::Ins(ins) => {
                self.addr += WORD_SIZE;
                match ins {
                    Ins::Branch(ins) => self.eval_target(&mut ins.target),
                    Ins::DoubleOperand(ins) => {
                        self.eval_operand(&mut ins.src);
                        self.eval_operand(&mut ins.dst);
                    }
                    Ins::Jmp(ins) => self.eval_operand(&mut ins.dst),
                    Ins::Jsr(ins) => self.eval_operand(&mut ins.dst),
                    Ins::SingleOperand(ins) => self.eval_operand(&mut ins.dst),
                    Ins::Eis(ins) => self.eval_operand(&mut ins.operand),
                    Ins::Trap(ins) => {
                        if let Ok(val) = self.eval_expr(&ins.data) {
                            assert_eq!(val.val & !0xff, 0);
                            ins.data = Expr::Atom(Atom::Val(val.val));
                        }
                    }
                    _ => (),
                }
            }
            Cmd::Bytes(exprs) => {
                for e in exprs {
                    if let Ok(val) = self.eval_expr(e) {
                        *e = Expr::Atom(Atom::Val(val.val));
                    }
                    self.addr += 1;
                    self.loc += 1;
                }
            }
            Cmd::Words(exprs) => {
                for e in exprs {
                    if let Ok(val) = self.eval_expr(e) {
                        *e = Expr::Atom(Atom::Val(val.val));
                    }
                    self.addr += WORD_SIZE;
                    self.loc += WORD_SIZE;
                }
            }
            Cmd::LocDef(expr) => {
                if let Ok(val) = self.eval_expr(expr) {
                    assert!(val.val >= self.addr);
                    self.addr = val.val;
                    *expr = Expr::Atom(Atom::Val(self.addr))
                }
            }
            Cmd::Even => {
                self.addr += self.addr & 0x1;
            }
            Cmd::Ascii(v) => {
                self.addr += v.len().to_u16p();
            }
        }
    }

    fn eval_pass(&mut self, prog: &mut [Stmt]) {
        self.tmp_symbols.clear();
        self.addr = 0;
        for (l, stmt) in prog.iter_mut().enumerate() {
            self.line = l + 1;

            match &stmt.label_def {
                Label::Regular(label) => {
                    let sym = SymbolValue::new(
                        SymbolType::Label,
                        Value::new(self.addr, self.sect.mode()),
                        self.line,
                    );
                    let existing = self.symbols.insert(label.clone(), sym);
                    if let Some(existing) = existing {
                        if existing.line != self.line {
                            panic!("Label '{label}' on line {} conflicts with previous definition on line {}", self.line, existing.line);
                        }
                    }
                }
                Label::Tmp(val) => {
                    let sym = SymbolValue::new(
                        SymbolType::Label,
                        Value::new(self.addr, self.sect.mode()),
                        self.line,
                    );
                    self.tmp_symbols.insert(*val, sym);
                    self.tmp_f_tracker.found(*val, self.addr, self.sect.mode());
                }
                Label::None => (),
            }

            self.loc = self.addr;
            if let Some(cmd) = stmt.cmd.as_mut() {
                self.eval_cmd(cmd);
            }
        }
    }

    const MAX_ITER: i32 = 2;
    fn eval_prog(&mut self, prog: &mut [Stmt]) {
        for _ in 1..=Self::MAX_ITER {
            self.eval_pass(prog);
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
            .map(|(x, i)| {
                parser.parse(x).map_err(|e| {
                    eprintln!("Error line {i}: {e}");
                    e
                })
            })
            .collect();

        let mut prog: Vec<_> = prog
            .into_iter()
            .map(|x| x.unwrap_or_else(|_| panic!("Exiting due to previous errors")))
            .filter(|x| !x.is_empty())
            .collect();

        self.eval_prog(&mut prog);
        self.check_resolved(&prog);

        for stmt in prog {
            stmt.emit(&mut self.buf);
        }
        Program {
            text: self.buf,
            symbols: self.symbols,
        }
    }
}

pub fn assemble(prog: &str) -> Aout {
    let prog = Assembler::new().assemble(prog);
    let start = prog.symbols.get("_start").expect("_start not found");
    let mut aout = Aout::empty();
    aout.text = prog.text;
    aout.entry_point = start.val;
    aout
}

// For testing
pub fn assemble_raw(prog: &str) -> Program {
    Assembler::new().assemble(prog)
}

#[cfg(test)]
mod tests {
    use super::assemble_raw;

    fn to_u16_vec(arr: &Vec<u8>) -> Vec<u16> {
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
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0);
    }

    #[test]
    fn mov_reg_reg() {
        let prog = "mov r0, r1";
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o10001);
    }

    #[test]
    fn mov_mem_mem() {
        let prog = "mov (r0)+, -(r1)";
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o12041);
    }

    #[test]
    fn branch_1() {
        let prog = r#"
            label:
                br label"#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o000777);
    }
    #[test]
    fn branch_2() {
        let prog = r#"
            label: br label"#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o000777);
    }

    #[test]
    fn branch_tmpb_1() {
        let prog = r#"
            1:
                br 1b"#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o000777);
    }

    #[test]
    fn branch_tmpb_2() {
        let prog = r#"
            1: br 1b"#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o000777);
    }

    #[test]
    fn branch_tmpf() {
        let prog = r#"
                br 34f
            34:"#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o400);
    }

    #[test]
    fn branch_tmpb_val_1() {
        let prog = r#"
            1:
                .word 0
            mov     1b, r0"#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[1], 0o016700);
        assert_eq!(bin[2], -6i16 as u16);
    }

    #[should_panic]
    #[test]
    fn branch_tmpb_never_defined_a() {
        let prog = r#"
                br 1b"#;
        assemble_raw(prog);
    }

    #[should_panic]
    #[test]
    fn branch_tmpb_never_defined_b() {
        let prog = r#"
                mov 23b, r0"#;
        assemble_raw(prog);
    }

    #[should_panic]
    #[test]
    fn branch_tmpf_never_defined_a() {
        let prog = r#"
                br 1f"#;
        assemble_raw(prog);
    }

    #[should_panic]
    #[test]
    fn branch_tmpf_never_defined_b() {
        let prog = r#"
                mov 534f, r0"#;
        assemble_raw(prog);
    }

    #[test]
    fn numbers() {
        let prog = r#"
            .word 0
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o0);

        let prog = r#"
            .word 7
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o7);

        let prog = r#"
            .word 17
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o17);

        let prog = r#"
            .word 0.
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0);

        let prog = r#"
            .word 7.
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 7);

        let prog = r#"
            .word 17.
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 17);
    }

    #[test]
    fn negative_numbers() {
        let prog = r#"
            .word -0
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o0);

        let prog = r#"
            .word -7
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], -0o7i16 as u16);

        let prog = r#"
            .word 17
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o17);

        let prog = r#"
            .word 0.
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0);

        let prog = r#"
            .word 7.
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 7);

        let prog = r#"
            .word 17.
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 17);
    }

    #[test]
    #[should_panic]
    fn byte_size() {
        let prog = r#"
            .byte 400
        "#;
        assemble_raw(prog).text;
    }

    #[test]
    #[should_panic]
    fn word_size() {
        let prog = r#"
            .word 200000
        "#;
        assemble_raw(prog).text;
    }

    #[test]
    fn char_literal() {
        let prog = r#"
            .byte 'A
        "#;
        let bin = assemble_raw(prog).text;
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0x41);
    }

    #[test]
    fn symbol() {
        let prog = r#"
            SYM = 37
            mov #SYM, r0
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[1], 0o37);
    }

    #[test]
    fn symbol_chain() {
        let prog = r#"
            a = 37
            b = a
            mov #b, r0
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
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
        let bin = to_u16_vec(&assemble_raw(prog).text);
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
        assemble_raw(prog);
    }

    #[test]
    #[should_panic]
    fn never_defined() {
        let prog = r#"
            a = b
            mov #a, r0
        "#;
        assemble_raw(prog);
    }

    #[test]
    fn symbol_byte() {
        let prog = r#"
            a = 37
            .byte a
        "#;
        let bin = assemble_raw(prog).text;
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o37);
    }

    #[test]
    fn symbol_word() {
        let prog = r#"
            a = 777
            .word a
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o777);
    }

    #[test]
    fn expr_word() {
        let prog = r#"
            .word 2 + 1
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o3);

        let prog = r#"
            .word 1 + 1 ! 2
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o2);

        let prog = r#"
            .word 1 ! 2 + 1
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o4);
    }

    #[test]
    fn expr_byte() {
        let prog = r#"
            .byte 2 + 1
        "#;
        let bin = &assemble_raw(prog).text;
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o3);

        let prog = r#"
            .byte 1 + 1 ! 2
        "#;
        let bin = &assemble_raw(prog).text;
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o2);

        let prog = r#"
            .byte 1 ! 2 + 1
        "#;
        let bin = &assemble_raw(prog).text;
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o4);
    }

    #[test]
    #[should_panic] // The manual says to truncate, I've chosen not to.
    fn expr_byte_overflow() {
        let prog = r#"
            .byte 377 + 1
        "#;
        let bin = &assemble_raw(prog).text;
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o0);
    }

    #[test]
    fn expr_index() {
        let prog = r#"
            FIELD_A = 2 + 2
            mov FIELD_A(r0), r1
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[1], 0o4);

        let prog = r#"
            FIELD_A = 4
            mov FIELD_A + 2(r0), r1
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[1], 0o6);
    }

    #[test]
    fn period_expr() {
        let prog = r#"
            .word .
        "#;
        let bin = &assemble_raw(prog).text;
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[0], 0o0);

        let prog = r#"
            .word ., .
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[0], 0o0);
        assert_eq!(bin[1], 0o2);

        let prog = r#"
            clr r0
            mov #., r0
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[2], 0o2);

        let prog = r#"
            .word 0, 0
            loc = .
            .word loc
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[2], 0o4);

        let prog = r#"
            .word 0, 0
            .word loc
            loc = .
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[2], 0o6);
    }

    #[test]
    fn period_assign() {
        let prog = r#"
            . = 12
        "#;
        let bin = &assemble_raw(prog).text;
        assert_eq!(bin.len(), 10);

        let prog = r#"
            . = 2
            mov #., r0
        "#;
        let bin = to_u16_vec(&assemble_raw(prog).text);
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[2], 2);
    }

    #[test]
    fn even() {
        let prog = r#"
            .byte 0
        "#;
        let bin = assemble_raw(prog).text;
        assert_eq!(bin.len(), 1);

        let prog = r#"
            .byte 0
            .even
        "#;
        let bin = assemble_raw(prog).text;
        assert_eq!(bin.len(), 2);

        let prog = r#"
            . = 11
        "#;
        let bin = assemble_raw(prog).text;
        assert_eq!(bin.len(), 9);

        let prog = r#"
            . = 11
            .even
        "#;
        let bin = assemble_raw(prog).text;
        assert_eq!(bin.len(), 10);
    }

    #[test]
    fn eis() {
        let bin = to_u16_vec(&assemble_raw(r#"mul r1, r0"#).text);
        assert_eq!(bin, [0o070001]);

        let bin = to_u16_vec(&assemble_raw(r#"div @(r2)+, r4"#).text);
        assert_eq!(bin, [0o071432]);

        let bin = to_u16_vec(&assemble_raw(r#"ash #23, r5"#).text);
        assert_eq!(bin, [0o072527, 0o23]);

        let bin = to_u16_vec(&assemble_raw(r#"label: ashc label, r5"#).text);
        assert_eq!(bin, [0o073567, 0o177774]);

        let bin = to_u16_vec(&assemble_raw(r#"label: xor label, r5"#).text);
        assert_eq!(bin, [0o074567, 0o177774]);
    }

    #[test]
    #[should_panic]
    fn div_odd() {
        assemble_raw(r#"div @(r2).text+, r5"#);
    }

    #[test]
    #[should_panic]
    fn label_redef_0() {
        assemble_raw(
            r#"
        label:
        label:
        "#,
        );
    }

    #[test]
    #[should_panic]
    fn label_redef_1() {
        assemble_raw(
            r#"
        label:
        label = 1
        "#,
        );
    }

    #[test]
    #[should_panic]
    fn label_redef_2() {
        assemble_raw(
            r#"
        label = 1
        label:
        "#,
        );
    }

    #[test]
    fn label_redef_3() {
        let prog = assemble_raw(
            r#"
        label = 1
        label = 2
        "#,
        );
        assert_eq!(prog.symbols.get("label").unwrap().val, 2);
    }
}
