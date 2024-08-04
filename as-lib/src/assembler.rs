 
use std::collections::HashMap;
use std::convert::TryInto;

use crate::ir::*;
use crate::grammar::StmtParser;
use common::asm::*;
use common::constants::WORD_SIZE;
use common::mem::as_byte_slice;

use num_traits::ToPrimitive;
use log::trace;

pub fn assemble(prog: &str) -> Vec<u8> {
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
                Cmd::Bytes(b) => self.buf.extend(b),
                Cmd::Words(words) => {
                    let is_le = || u16::from_ne_bytes([1, 0]) == 1;
                    assert!(is_le(), "Only little-endian hosts supported");
                    self.buf.extend(as_byte_slice(words.as_slice()));
                },
                Cmd::Ascii(a) => self.buf.extend(a),
                Cmd::Ins(ins) => self.emit_ins(ins),
                Cmd::SymbolDef(_, _) => (),
            }
        }
    }

    fn emit(&mut self, word: u16) {
        let lower = word as u8;
        let upper = (word >> 8) as u8;
        self.buf.push(lower);
        self.buf.push(upper);
    }

    fn eval_expr(&self, expr: &Expr, iter: i32) -> Option<u16> {
        match expr {
            Expr::Val(val) => Some(*val),
            Expr::SymbolRef(symbol) => {
                let val = self.symbols.get(symbol).cloned();
                if val.is_some() {
                    val
                } else if iter == Self::MAX_ITER {
                    panic!("Can't resolve {}", expr.clone().unwrap_symbol_ref());
                } else {
                    None
                }
            }
        }
    }

    fn resolve_operand(&self, arg: &mut Operand, curr_addr: &mut u16, iter: i32) {
        let loc = match &arg.extra {
            Extra::None => return,
            Extra::Imm(expr) => {
                let loc = self.eval_expr(expr, iter);
                if let (Expr::SymbolRef(symbol), Some(loc)) = (expr, loc) {
                    trace!("Resolving symbol \"{symbol}\" (imm) to loc 0o{loc:o}, curr_addr: 0o{curr_addr:o}");
                }
                loc
            },
            Extra::Rel(expr) => {
                self.eval_expr(expr, iter).map(|val| {
                    let loc = (val as i32 - *curr_addr as i32 - 2) as u16;

                    if let Expr::SymbolRef(symbol) = expr {
                        trace!("Resolving symbol \"{symbol}\" (rel) to loc 0o{loc:o}, curr_addr: 0o{curr_addr:o}, final: 0o{val:o}");
                    }
                    loc
                })
            }
        };

        if let Some(loc) = loc {
            arg.extra = Extra::Imm(Expr::Val(loc));
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

    fn resolve_symbols(&mut self, prog: &mut [Stmt]) {
        for iter in 1..=Self::MAX_ITER {
            let mut addr: u16 = 0;
            for stmt in prog.iter_mut() {

                if let Some(label) = &stmt.label_def {
                    self.symbols.insert(label.clone(), addr); 
                }

                if stmt.cmd.is_none() {
                    continue;
                }

                match stmt.cmd.as_mut().unwrap() {
                    Cmd::SymbolDef(symbol, expr) => {
                        if let Some(val) = self.eval_expr(expr, iter) {
                            self.symbols.insert(symbol.clone(), val);
                        }
                    },
                    Cmd::Ins(ins) => {
                        match ins {
                            Ins::Branch(ins) => self.resolve_target(&mut ins.target, addr, iter),
                            Ins::DoubleOperand(ins) => {
                                self.resolve_operand(&mut ins.src, &mut addr, iter);
                                self.resolve_operand(&mut ins.dst, &mut addr, iter);
                            },
                            Ins::Jmp(ins) => self.resolve_operand(&mut ins.dst, &mut addr, iter),
                            Ins::Jsr(ins) => self.resolve_operand(&mut ins.dst, &mut addr, iter),
                            Ins::SingleOperand(ins) => self.resolve_operand(&mut ins.dst, &mut addr, iter),
                            _ => (),
                        }
                        addr += WORD_SIZE;
                    }
                    _ => { addr += stmt.size(); },
                }
            }
        }
    }

    fn assemble(mut self, prog: &str) -> Vec<u8> {

        let lines = prog.split('\n');
        let parser = StmtParser::new();

        let mut prog: Vec<Stmt> = lines
            .zip(1..)
            .map(|(x,i)| {
                parser.parse(x).unwrap_or_else(|e| panic!("Error line {}: {}", i, e))
            })
            .filter(|x| !x.is_empty())
            .collect();

        self.resolve_symbols(&mut prog);

        for stmt in prog {
            self.emit_stmt(&stmt);
        }
        self.buf
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
}






