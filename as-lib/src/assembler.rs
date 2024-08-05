 
use std::collections::HashMap;
use std::convert::TryInto;

use crate::ir::*;
use crate::grammar::StmtParser;
use common::asm::*;
use common::constants::WORD_SIZE;

use num_traits::ToPrimitive;
use log::trace;

pub fn assemble(prog: &str) -> Vec<u8> {
    Assembler::new().assemble(prog).0
}

pub fn assemble_with_symbols(prog: &str) -> (Vec<u8>, HashMap<String, u16>) {
    Assembler::new().assemble(prog)
}


struct Assembler {
    buf: Vec<u8>,
    symbols: HashMap<String, u16>,
}


impl Assembler {

    fn new() -> Assembler {
        Assembler{
            buf: Vec::new(),
            symbols: HashMap::new(),
        }
    }

    fn emit_double_operand_ins(&mut self, ins: &DoubleOperandIns) {
        let bin = (ins.op.to_u16().unwrap() <<DoubleOperandIns::LOWER_BITS) 
            | (ins.src.format() << Operand::NUM_BITS) 
            | ins.dst.format();
        self.emit(bin);

        if ins.src.has_imm() {
            self.emit(ins.src.extra.unwrap_val());
        }

        if ins.dst.has_imm() {
            self.emit(ins.dst.extra.unwrap_val());
        }
    }

    fn emit_branch_ins(&mut self, ins: &BranchIns) {
        let offset = ins.target.unwrap_offset();
        let bin = (ins.op.to_u16().unwrap() << BranchIns::LOWER_BITS) | (offset as u16);
        self.emit(bin);
    }

    fn emit_jmp_ins(&mut self, ins: &JmpIns) {
        let bin = (ins.op.to_u16().unwrap() << JmpIns::LOWER_BITS) | ins.dst.format();
        self.emit(bin);
        if ins.dst.has_imm() {
            self.emit(ins.dst.extra.unwrap_val());
        }
    }

    fn emit_jsr_ins(&mut self, ins: &JsrIns) {
        let bin = (ins.op.to_u16().unwrap() << JsrIns::LOWER_BITS)
            | (ins.reg.to_u16().unwrap() << Operand::NUM_BITS)
            | ins.dst.format();
        self.emit(bin);
        if ins.dst.has_imm() {
            self.emit(ins.dst.extra.unwrap_val());
        }
    }

    fn emit_rts_ins(&mut self, ins: &RtsIns) {
        let bin = (ins.op.to_u16().unwrap() << RtsIns::LOWER_BITS) | ins.reg.to_u16().unwrap();
        self.emit(bin);
    }

    fn emit_single_operand_ins(&mut self, ins: &SingleOperandIns) {
        let bin = (ins.op.to_u16().unwrap() << SingleOperandIns::LOWER_BITS) | ins.dst.format();
        self.emit(bin);
        if ins.dst.has_imm() {
            self.emit(ins.dst.extra.unwrap_val());
        }

    }
    fn emit_cc_ins(&mut self, ins: &CCIns) {
        self.emit(ins.op.to_u16().unwrap());
    }
    fn emit_misc_ins(&mut self, ins: &MiscIns) {
        self.emit(ins.op.to_u16().unwrap());
    }

    fn emit_trap_ins(&mut self, ins: &TrapIns) {
        let offset = ins.handler.unwrap_offset();
        self.emit((ins.op.to_u16().unwrap() << TrapIns::LOWER_BITS) | (offset as u16));
    }

    fn emit_ins(&mut self, ins: &Ins) {
        match ins {
            Ins::DoubleOperand(x) => self.emit_double_operand_ins(x),
            Ins::Branch(x) => self.emit_branch_ins(x),
            Ins::Jmp(x) => self.emit_jmp_ins(x),
            Ins::Jsr(x) => self.emit_jsr_ins(x),
            Ins::Rts(x) => self.emit_rts_ins(x),
            Ins::SingleOperand(x) => self.emit_single_operand_ins(x),
            Ins::CC(x) => self.emit_cc_ins(x),
            Ins::Misc(x) => self.emit_misc_ins(x),
            Ins::Trap(x) => self.emit_trap_ins(x),
        }
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        if let Some(cmd) = &stmt.cmd {
            match cmd {
                Cmd::Bytes(exprs) => {
                    for e in exprs {
                        let val = TryInto::<u8>::try_into(e.unwrap_val()).unwrap();
                        self.buf.push(val);
                    }
                }
                Cmd::Words(exprs) => {
                    for e in exprs {
                        self.emit(e.unwrap_val());
                    }
                },
                Cmd::Ascii(a) => self.buf.extend(a),
                Cmd::Ins(ins) => self.emit_ins(ins),
                Cmd::SymbolDef(_, _) => (),
                Cmd::LocDef(addr) => {
                    let addr = addr.unwrap_val();
                    assert!(addr as usize >= self.buf.len());
                    self.buf.resize(addr as usize, 0);
                    
                },
                Cmd::Even => {
                    if self.buf.len() & 0x1 == 1 {
                        self.buf.push(0);
                    }
                }
            }
        }
    }

    fn emit(&mut self, word: u16) {
        let lower = word as u8;
        let upper = (word >> 8) as u8;
        self.buf.push(lower);
        self.buf.push(upper);
    }

    fn eval_atom(&self, atom: &Atom, loc: u16, iter: i32) -> Option<u16> {
        match atom {
            Atom::Loc => Some(loc),
            Atom::Val(val) => Some(*val),
            Atom::SymbolRef(symbol) => {
                let val = self.symbols.get(symbol).cloned();
                if val.is_some() {
                    val
                } else if iter == Self::MAX_ITER {
                    panic!("Can't resolve {}", atom.clone().unwrap_symbol_ref());
                } else {
                    None
                }
            }
        }
    }

    fn eval_expr(&self, expr: &Expr, loc: u16, iter: i32) -> Option<u16> {
        match expr {
            Expr::Atom(atom) => self.eval_atom(atom, loc, iter),
            Expr::Op(lhs, op, rhs) => {
                let lhs = self.eval_expr(lhs, loc, iter);
                let rhs = self.eval_atom(rhs, loc, iter);
                let (Some(lhs), Some(rhs)) = (lhs, rhs) else {
                    return None;
                };

                match op {
                    Op::Add => Some(lhs.wrapping_add(rhs)),
                    Op::Sub => Some(lhs.wrapping_sub(rhs)),
                    Op::And => Some(lhs & rhs),
                    Op::Or => Some(lhs | rhs),
                }
            },
        }
    }

    fn resolve_operand(&self, arg: &mut Operand, curr_addr: &mut u16, loc: u16, iter: i32) {
        let val = match &arg.extra {
            Extra::None => return,
            Extra::Imm(expr) => {
                let val = self.eval_expr(expr, loc, iter);
                if let (Expr::Atom(Atom::SymbolRef(symbol)), Some(val)) = (expr, val) {
                    trace!("Resolving symbol \"{symbol}\" (imm) to val 0o{val:o}, curr_addr: 0o{curr_addr:o}");
                }
                val
            },
            Extra::Rel(expr) => {
                self.eval_expr(expr, loc, iter).map(|val| {
                    let off = (val as i32 - *curr_addr as i32 - 2) as u16;

                    if let Expr::Atom(Atom::SymbolRef(symbol)) = expr {
                        trace!("Resolving symbol \"{symbol}\" (rel) to offset 0o{off:o}, curr_addr: 0o{curr_addr:o}, final: 0o{val:o}");
                    }
                    off
                })
            }
        };

        if let Some(val) = val {
            arg.extra = Extra::Imm(Expr::Atom(Atom::Val(val)));
        }
        *curr_addr += WORD_SIZE;
    }

    fn resolve_target(&self, target: &mut Target, curr_addr: u16, iter: i32) {
        let offset = match target {
            Target::Offset(x) => *x,
            Target::Label(ref label) => {
                if let Some(dst) = self.symbols.get(label) {
                    let addr = curr_addr as i32;
                    TryInto::<i8>::try_into((*dst as i32 - addr - 2)/2).unwrap() as u8
                } else if iter == Self::MAX_ITER {
                    panic!("Label {} not found", label)
                } else {
                    return
                }
            },
        };
        *target = Target::Offset(offset);
    }

    const MAX_ITER: i32 = 2;

    fn resolve_and_eval(&mut self, prog: &mut [Stmt]) {
        for iter in 1..=Self::MAX_ITER {
            let mut addr: u16 = 0;
            for stmt in prog.iter_mut() {

                if let Some(label) = &stmt.label_def {
                    self.symbols.insert(label.clone(), addr); 
                }

                if stmt.cmd.is_none() {
                    continue;
                }

                let loc = addr;
                match stmt.cmd.as_mut().unwrap() {
                    Cmd::SymbolDef(symbol, expr) => {
                        if let Some(val) = self.eval_expr(expr, loc, iter) {
                            self.symbols.insert(symbol.clone(), val);
                        }
                    },
                    Cmd::Ins(ins) => {
                        match ins {
                            Ins::Branch(ins) => self.resolve_target(&mut ins.target, addr, iter),
                            Ins::DoubleOperand(ins) => {
                                self.resolve_operand(&mut ins.src, &mut addr, loc, iter);
                                self.resolve_operand(&mut ins.dst, &mut addr, loc, iter);
                            },
                            Ins::Jmp(ins) => self.resolve_operand(&mut ins.dst, &mut addr, loc, iter),
                            Ins::Jsr(ins) => self.resolve_operand(&mut ins.dst, &mut addr, loc, iter),
                            Ins::SingleOperand(ins) => self.resolve_operand(&mut ins.dst, &mut addr, loc, iter),
                            _ => (),
                        }
                        addr += WORD_SIZE;
                    }
                    Cmd::Bytes(exprs) => {
                        for e in exprs {
                            if let Some(val) = self.eval_expr(e, addr, iter) {
                                *e = Expr::Atom(Atom::Val(val));
                            }
                            addr += 1;
                        }
                    }
                    Cmd::Words(exprs) => {
                        for e in exprs {
                            if let Some(val) = self.eval_expr(e, addr, iter) {
                                *e = Expr::Atom(Atom::Val(val));
                            }
                            addr += WORD_SIZE;
                        }
                    },
                    Cmd::LocDef(expr) => {
                        if let Some(val) = self.eval_expr(expr, addr, iter) {
                            assert!(val >= addr);
                            addr = val;
                            *expr = Expr::Atom(Atom::Val(addr))
                        }
                    },
                    Cmd::Even => {
                        if addr & 0x1 == 1 {
                            addr += 1;
                        }
                    },
                    _ => { addr += stmt.size().unwrap(); },
                }
            }
        }
    }

    fn assemble(mut self, prog: &str) -> (Vec<u8>, HashMap<String, u16>) {

        let lines = prog.split('\n');
        let parser = StmtParser::new();

        let mut prog: Vec<Stmt> = lines
            .zip(1..)
            .map(|(x,i)| {
                parser.parse(x).unwrap_or_else(|e| panic!("Error line {}: {}", i, e))
            })
            .filter(|x| !x.is_empty())
            .collect();

        self.resolve_and_eval(&mut prog);

        for stmt in prog {
            self.emit_stmt(&stmt);
        }
        (self.buf, self.symbols)
    }
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
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0);
    }

    #[test]
    fn mov_reg_reg() {
        let prog = "mov r0, r1";
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o10001);
    }

    #[test]
    fn mov_mem_mem() {
        let prog = "mov (r0)+, -(r1)";
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o12041);
    }

    #[test]
    fn branch() {
        let prog = r#"
            label:
                br label"#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o000777);
    }

    #[test]
    fn numbers() {
        let prog = r#"
            .word 0
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o0);

        let prog = r#"
            .word 7
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o7);

        let prog = r#"
            .word 17
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o17);

        let prog = r#"
            .word 0.
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0);

        let prog = r#"
            .word 7.
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 7);

        let prog = r#"
            .word 17.
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 17);
    }

    #[test]
    fn negative_numbers() {
        let prog = r#"
            .word -0
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o0);

        let prog = r#"
            .word -7
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], -0o7i16 as u16);

        let prog = r#"
            .word 17
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o17);

        let prog = r#"
            .word 0.
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0);

        let prog = r#"
            .word 7.
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 7);

        let prog = r#"
            .word 17.
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 17);
    }

    #[test]
    #[should_panic]
    fn byte_size() {
        let prog = r#"
            .byte 400
        "#;
        assemble(prog);
    }

    #[test]
    #[should_panic]
    fn word_size() {
        let prog = r#"
            .word 200000
        "#;
        assemble(prog);
    }

    #[test]
    fn char_literal() {
        let prog = r#"
            .byte 'A
        "#;
        let bin = assemble(prog);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0x41);
    }

    #[test]
    fn symbol() {
        let prog = r#"
            SYM = 37
            mov #SYM, r0
        "#;
        let bin = to_u16(&assemble(prog));
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
        let bin = to_u16(&assemble(prog));
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
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[1], 0o37);
    }

    #[test]
    fn symbol_byte() {
        let prog = r#"
            a = 37
            .byte a
        "#;
        let bin = assemble(prog);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o37);
    }

    #[test]
    fn symbol_word() {
        let prog = r#"
            a = 777
            .word a
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o777);
    }

    #[test]
    fn expr_word() {
        let prog = r#"
            .word 2 + 1
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o3);

        let prog = r#"
            .word 1 + 1 ! 2
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o2);

        let prog = r#"
            .word 1 ! 2 + 1
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o4);
    }

    #[test]
    fn expr_byte() {
        let prog = r#"
            .byte 2 + 1
        "#;
        let bin = &assemble(prog);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o3);

        let prog = r#"
            .byte 1 + 1 ! 2
        "#;
        let bin = &assemble(prog);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o2);

        let prog = r#"
            .byte 1 ! 2 + 1
        "#;
        let bin = &assemble(prog);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o4);
    }


    #[test]
    #[should_panic] // The manual says to truncate, I've chosen not to.
    fn expr_byte_overflow() {
        let prog = r#"
            .byte 377 + 1
        "#;
        let bin = &assemble(prog);
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o0);
    }

    #[test]
    fn expr_index() {
        let prog = r#"
            FIELD_A = 2 + 2
            mov FIELD_A(r0), r1
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[1], 0o4);

        let prog = r#"
            FIELD_A = 4
            mov FIELD_A + 2(r0), r1
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[1], 0o6);
    }

    #[test]
    fn period_expr() {
        let prog = r#"
            .word .
        "#;
        let bin = &assemble(prog);
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[0], 0o0);

        let prog = r#"
            .word ., .
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 2);
        assert_eq!(bin[0], 0o0);
        assert_eq!(bin[1], 0o2);

        let prog = r#"
            clr r0
            mov #., r0
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[2], 0o2);

        let prog = r#"
            .word 0, 0
            loc = .
            .word loc
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[2], 0o4);

        let prog = r#"
            .word 0, 0
            .word loc
            loc = .
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[2], 0o6);
    }


    #[test]
    fn period_assign() {
        let prog = r#"
            . = 12
        "#;
        let bin = &assemble(prog);
        assert_eq!(bin.len(), 10);

        let prog = r#"
            . = 2
            mov #., r0
        "#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 3);
        assert_eq!(bin[2], 2);
    }

    #[test]
    fn even() {
        let prog = r#"
            .byte 0
        "#;
        let bin = assemble(prog);
        assert_eq!(bin.len(), 1);

        let prog = r#"
            .byte 0
            .even
        "#;
        let bin = assemble(prog);
        assert_eq!(bin.len(), 2);

        let prog = r#"
            . = 11
        "#;
        let bin = assemble(prog);
        assert_eq!(bin.len(), 9);

        let prog = r#"
            . = 11
            .even
        "#;
        let bin = assemble(prog);
        assert_eq!(bin.len(), 10);
    }
}






