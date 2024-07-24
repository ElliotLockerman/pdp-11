 
use std::collections::HashMap;
use std::convert::TryInto;

use crate::ir::*;
use crate::grammar::StmtParser;
use common::asm::*;
use common::mem::as_byte_slice;

use num_traits::ToPrimitive;

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
            | (ins.src.format() << RegArg::NUM_BITS) 
            | ins.dst.format();
        self.emit(bin);

        if ins.src.has_imm() {
            self.emit(ins.src.extra.unwrap_imm());
        }

        if ins.dst.has_imm() {
            self.emit(ins.dst.extra.unwrap_imm());
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
            self.emit(ins.dst.extra.unwrap_imm());
        }
    }

    fn emit_jsr_ins(&mut self, ins: &JsrIns) {
        let bin = (ins.op.to_u16().unwrap() << JsrIns::LOWER_BITS)
            | (ins.reg.to_u16().unwrap() << RegArg::NUM_BITS)
            | ins.dst.format();
        self.emit(bin);
        if ins.dst.has_imm() {
            self.emit(ins.dst.extra.unwrap_imm());
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
            self.emit(ins.dst.extra.unwrap_imm());
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
                Cmd::Words(words) => self.buf.extend(unsafe { as_byte_slice(words.as_slice()) }),
                Cmd::Ascii(a) => self.buf.extend(a),
                Cmd::Ins(ins) => self.emit_ins(ins),

            }
        }
    }

    fn emit(&mut self, word: u16) {
        let lower = word as u8;
        let upper = (word >> 8) as u8;
        self.buf.push(lower);
        self.buf.push(upper);
    }

    fn resolve_regarg(&self, arg: &mut RegArg, curr_addr: u16) {
        if !arg.extra.is_label_ref() {
            return;
        }
        // TODO: switch to relative/deferred relative (index/indexdef pc)
        if matches!(arg.mode, AddrMode::Index | AddrMode::IndexDef) {
            assert_eq!(arg.reg, Reg::PC);

            let extra = arg.extra.take();
            let loc = self.symbols.get(extra.unwrap_label_ref()).unwrap();
            arg.extra = Extra::Imm((*loc as i32 - curr_addr as i32 - 2) as u16);
            println!("Resolving label \"{}\" to loc {loc}, curr_addr: {curr_addr}, final: {:?}", extra.unwrap_label_ref(), arg.extra);

        } else if matches!(arg.mode, AddrMode::AutoInc) {
            let extra = arg.extra.take();
            let loc = self.symbols.get(extra.unwrap_label_ref()).unwrap();
            arg.extra = Extra::Imm(*loc);
        }
    }

    fn resolve_target(&self, target: &mut Target, curr_addr: u16) {
        let offset = match target {
            Target::Offset(x) => *x,
            Target::Label(ref label) => {
                if let Some(dst) = self.symbols.get(label) {
                    let addr = curr_addr as i32;
                    TryInto::<i8>::try_into((*dst as i32 - addr - 2)/2).unwrap() as u8
                } else {
                    panic!("Label {} not found", label)
                }
            },
        };
        *target = Target::Offset(offset);
    }

    fn resolve_symbols(&mut self, prog: &mut [Stmt]) {
        let mut addr: u16 = 0;
        for stmt in prog.iter() {
            if let Some(label) = &stmt.label_def {
                self.symbols.insert(label.clone(), addr); 
            }
            addr += stmt.size();
        }

        addr = 0;
        for stmt in prog.iter_mut() {
            if let Some(Cmd::Ins(ins)) = &mut stmt.cmd {
                match ins {
                    Ins::Branch(ins) => self.resolve_target(&mut ins.target, addr),
                    Ins::DoubleOperand(ins) => {
                        self.resolve_regarg(&mut ins.src, addr);
                        self.resolve_regarg(&mut ins.dst, addr);
                    },
                    Ins::Jmp(ins) => self.resolve_regarg(&mut ins.dst, addr),
                    Ins::Jsr(ins) => self.resolve_regarg(&mut ins.dst, addr),

                    // TODO: other kinds of labels!
                    _ => (),
                }
            }
            addr += stmt.size();
        }
    }

    fn assemble(mut self, prog: &str) -> Vec<u8> {

        let lines = prog.split('\n');
        let parser = StmtParser::new();

        let mut prog: Vec<Stmt> = lines
            .zip(1..)
            .filter(|(x,_)| !x.is_empty())
            .map(|(x,i)| {
                parser.parse(x).unwrap_or_else(|e| panic!("Error line {}: {}", i, e))
            })
            .collect();

        self.resolve_symbols(&mut prog);

        for stmt in prog {
            self.emit_stmt(&stmt);
        }
        self.buf
    }
}


















