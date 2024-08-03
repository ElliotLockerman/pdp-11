
use common::asm::*;
use common::decoder::decode;
use common::constants::*;
use crate::MMIOHandler;
use crate::EmulatorState;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::{BitOr, BitAnd};
use std::rc::Rc;
use std::cell::RefCell;

use log::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Size {
    Byte,
    Word,
}

impl Size {
    fn bytes(self) -> u16 {
        match self {
            Size::Byte => 1,
            Size::Word => 2,
        }
    }

    fn bits(&self) -> u16 {
        self.bytes() * 8
    }

    fn mask(self) -> u32 {
        match self {
            Size::Byte => 0xff,
            Size::Word => 0xffff,
        }
    }

    fn smallest(self) -> u32 {
        0x1 << (self.bits() - 1)
    }

    fn largest(self) -> u32 {
        self.smallest().wrapping_sub(1)
    }
}

fn sign_bit(n: u32, size: Size) -> u32 {
    match size {
        Size::Word => (n >> 15) & 0x1,
        Size::Byte => (n >> 7) & 0x1,
    }
}

#[derive(Debug, Clone, Copy)]
enum ResolvedRegArg {
    Reg(Reg),
    Mem(u16),
}



#[derive(Debug, Clone, Copy)]
enum ExecRet {
    Ok,
    Jmp,
    Halt,
}


pub struct Emulator {
    state: EmulatorState,
    mmio_handlers: HashMap<u16, Rc<RefCell<dyn MMIOHandler>>>,
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            state: EmulatorState::new(),
            mmio_handlers: HashMap::new(),
        }
    }
    pub fn run(&mut self) {
        loop {
            // TODO: track actual time instead of assuming 1 cycle / ins
            self.state.inc_cycle();
            for handler in self.mmio_handlers.values_mut() { 
                handler.borrow_mut().cycle(&mut self.state); 
            }
            
            let ins = self.decode();
            debug!("PC: 0{:0}: {:?}", self.state.pc(), ins);
            self.state.reg_write_word(Reg::PC, self.state.pc() + 2);
            match self.exec(&ins) {
                ExecRet::Ok => (),
                ExecRet::Jmp => (),
                ExecRet::Halt => return,
            }
        }
    }

    fn decode(&self) -> Ins {
        decode(self.state.next_ins())
    }

    pub fn run_at(&mut self, pc: u16) {
        self.state.reg_write_word(Reg::PC, pc);
        self.run();
    }

    pub fn load_image(&mut self, data: &[u8], start: u16) {
        let end = start + u16::try_from(data.len()).unwrap();
        for (byte, ptr) in data.iter().zip(start..end) {
            self.mem_write_byte(ptr, *byte);
        }
    }

    pub fn set_mmio_handler<I, M>(&mut self, addrs: I, handler: M) 
    where 
        I: IntoIterator<Item = u16>,
        M: MMIOHandler + 'static {

        let handler = Rc::new(RefCell::new(handler));
        for addr in addrs.into_iter() {
            assert!(addr >= MMIO_START);
            assert!(addr & 0x1 == 0, "MMIOHandler addr {addr:o} not aligned");
            let prev = self.mmio_handlers.insert(addr, handler.clone());
            assert!(prev.is_none(), "Duplicate MMIOHandler for {addr:o}");
        }
    }

    ///////////////////////////////////////////////////////////////////////////


    pub fn mem_read_byte(&mut self, addr: u16) -> u8 {
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                return handler.borrow_mut().read_byte(&mut self.state, addr);
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.state.mem_read_byte(addr)
        }
    }

    pub fn mem_write_byte(&mut self, addr: u16, val: u8) {
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                handler.borrow_mut().write_byte(&mut self.state, addr, val);
                return;
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.state.mem_write_byte(addr, val)
        }
    }

    pub fn mem_read_word(&mut self, addr: u16) -> u16 {
        assert!(addr & 1 == 0);
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                return handler.borrow_mut().read_word(&mut self.state, addr);
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.state.mem_read_word(addr)
        }
    }

    pub fn mem_write_word(&mut self, addr: u16, val: u16) {
        assert!(addr & 1 == 0);
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                handler.borrow_mut().write_word(&mut self.state, addr, val);
                return;
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.state.mem_write_word(addr, val)
        }
    }

    pub fn get_state(&self) -> &EmulatorState {
        &self.state
    }

    pub fn get_state_mut(&mut self) -> &mut EmulatorState {
        &mut self.state
    }

    fn write_resolved_word(&mut self, res: ResolvedRegArg, val: u16) {
        match res {
            ResolvedRegArg::Reg(r) => self.state.reg_write_word(r, val),
            ResolvedRegArg::Mem(addr) => self.mem_write_word(addr, val),
        }
    }

    fn write_resolved_byte(&mut self, res: ResolvedRegArg, val: u8) {
        match res {
            ResolvedRegArg::Reg(r) => self.state.reg_write_byte(r, val),
            ResolvedRegArg::Mem(addr) => self.mem_write_byte(addr, val),
        }
    }
    fn read_resolved_byte(&mut self, res: ResolvedRegArg) -> u8 {
        match res {
            ResolvedRegArg::Reg(r) => self.state.reg_read_byte(r),
            ResolvedRegArg::Mem(addr) => self.mem_read_byte(addr),
        }
    }
    fn read_resolved_word(&mut self, res: ResolvedRegArg) -> u16 {
        match res {
            ResolvedRegArg::Reg(r) => self.state.reg_read_word(r),
            ResolvedRegArg::Mem(addr) => self.mem_read_word(addr),
        }
    }

    fn read_resolved_widen(&mut self, res: ResolvedRegArg, size: Size) -> u32 {
        match size {
            Size::Word => self.read_resolved_word(res) as u32,
            Size::Byte => self.read_resolved_byte(res) as u32,
        }
    }

    fn write_resolved_narrow(&mut self, res: ResolvedRegArg, val: u32, size: Size) {
        match size {
            Size::Word => self.write_resolved_word(res, val as u16),
            Size::Byte => self.write_resolved_byte(res, val as u8),
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Execute
    ///////////////////////////////////////////////////////////////////////////
    // Returns the address, not the value
    fn exec_auto(&mut self, reg: Reg, inc: bool, mut size: Size) -> u16 {
        if reg == Reg::PC {
            // Special case for literals for byte instructions
            size = Size::Word;
        }
        let mut val = self.state.reg_read_word(reg);
        if !inc { 
            val -= size.bytes();
        }
        let ret = val;
        if inc { 
            val += size.bytes();
        }
        self.state.reg_write_word(reg, val);
        ret
    }


    fn resolve(&mut self, arg: &RegArg, size: Size) -> ResolvedRegArg {
        let loc = match arg.mode {
            AddrMode::Gen => return ResolvedRegArg::Reg(arg.reg),
            AddrMode::Def => self.state.reg_read_word(arg.reg),
            AddrMode::AutoInc => self.exec_auto(arg.reg, true, size),
            AddrMode::AutoIncDef => {
                let addr = self.exec_auto(arg.reg, true, Size::Word);
                self.mem_read_word(addr)
            },
            AddrMode::AutoDec => self.exec_auto(arg.reg, false, size),
            AddrMode::AutoDecDef => {
                let addr = self.exec_auto(arg.reg, false, Size::Word);
                self.mem_read_word(addr)
            },
            AddrMode::Index => {
                let reg_val = self.state.reg_read_word(arg.reg);
                let imm_addr = self.exec_auto(Reg::PC, true, Size::Word);
                let imm = self.mem_read_word(imm_addr);
                reg_val.wrapping_add(imm)
            },
            AddrMode::IndexDef => {
                let reg_val = self.state.reg_read_word(arg.reg);
                let imm_addr = self.exec_auto(Reg::PC, true, Size::Word);
                let imm = self.mem_read_word(imm_addr);
                self.mem_read_word(reg_val.wrapping_add(imm))
            },
        };

        ResolvedRegArg::Mem(loc)
    }

    fn do_mov(&mut self, src: &RegArg, dst: &RegArg, size: Size) {
        let src = self.resolve(src, size);
        let val = self.read_resolved_widen(src, size);
        let dst = self.resolve(dst, size);

        if size == Size::Byte {
            if matches!(dst, ResolvedRegArg::Reg(_)) {
                let val = val as u8 as i8 as i16 as u16;
                self.write_resolved_word(dst, val);
            } else {
                self.write_resolved_narrow(dst, val, size);
            }
        } else {
            self.write_resolved_word(dst, val as u16);
        }
        self.state.status.set_zero(val == 0);
        self.state.status.set_negative(sign_bit(val, size) != 0);
        self.state.status.set_overflow(false);
    }

    // TODO: combine these?
    fn do_bitwise(&mut self, src: &RegArg, op: fn(u32, u32) -> u32, dst: &RegArg, size: Size, discard: bool) {
        let src = self.resolve(src, Size::Word);
        let src_val = self.read_resolved_widen(src, size);
        let dst = self.resolve(dst, size);
        let dst_val = self.read_resolved_widen(dst, size);
        let res = op(src_val, dst_val);
        let res_sign = sign_bit(res, size);

        self.state.status.set_zero(res == 0);
        self.state.status.set_negative(res_sign != 0);
        // Carry not affected
        self.state.status.set_overflow(false);

        if !discard {
            self.write_resolved_narrow(dst, res, size);
        }
    }

    fn do_add(&mut self, src: &RegArg, dst: &RegArg, size: Size) {
        assert!(size == Size::Word);
        let src = self.resolve(src, size);
        let src_val = self.read_resolved_widen(src, size);
        let src_sign = sign_bit(src_val, size);
        let dst = self.resolve(dst, size);
        let dst_val = self.read_resolved_widen(dst, size);
        let dst_sign = sign_bit(dst_val, size);
        let res = src_val + dst_val;
        let res_sign = sign_bit(res, size);

        self.state.status.set_zero((res & size.mask()) == 0);
        self.state.status.set_negative(res_sign != 0);
        self.state.status.set_carry(res >> size.bits() != 0);
        self.state.status.set_overflow(src_sign == dst_sign && dst_sign != res_sign);
        self.write_resolved_narrow(dst, res, size);
    }

    // NB: src and dst are flipped for cmp
    fn do_sub(&mut self, src: &RegArg, dst: &RegArg, size: Size, discard: bool) {
        let src = self.resolve(src, size);
        let src_val = self.read_resolved_widen(src, size);
        let src_sign = sign_bit(src_val, size);
        let dst = self.resolve(dst, size);
        let dst_val = self.read_resolved_widen(dst, size);
        let dst_sign = sign_bit(dst_val, size);
        let res = dst_val.wrapping_add((!src_val).wrapping_add(1) & size.mask());
        let res_sign = sign_bit(res, size);

        self.state.status.set_zero((res & size.mask()) == 0);
        self.state.status.set_negative(res_sign != 0);
        self.state.status.set_carry(dst_val < src_val);
        self.state.status.set_overflow(src_sign != dst_sign && src_sign == res_sign);

        if !discard {
            self.write_resolved_narrow(dst, res, size);
        }
    }

    fn exec_double_operand_ins(&mut self, ins: &DoubleOperandIns) {
        use DoubleOperandOpcode::*;
        match ins.op {
            Mov => self.do_mov(&ins.src, &ins.dst, Size::Word),
            Cmp => self.do_sub(&ins.dst, &ins.src, Size::Word, true),
            Bis => self.do_bitwise(&ins.src, u32::bitor, &ins.dst, Size::Word, false),
            Bic => self.do_bitwise(&ins.src, not_and, &ins.dst, Size::Word, false),
            Bit => self.do_bitwise(&ins.src, u32::bitand, &ins.dst, Size::Word, true),

            Add => self.do_add(&ins.src, &ins.dst, Size::Word) ,

            MovB => self.do_mov(&ins.src, &ins.dst, Size::Byte),
            CmpB => self.do_sub(&ins.dst,  &ins.src, Size::Byte, true),
            BisB => self.do_bitwise(&ins.src, u32::bitor, &ins.dst, Size::Byte, false),
            BicB => self.do_bitwise(&ins.src, not_and, &ins.dst, Size::Byte, false),
            BitB => self.do_bitwise(&ins.src, u32::bitand, &ins.dst, Size::Byte, true),

            Sub => self.do_sub(&ins.src, &ins.dst, Size::Word, false) ,
        }
    }

    fn exec_misc_ins(&mut self, ins: &MiscIns) -> ExecRet {
        match ins.op {
            MiscOpcode::Halt => ExecRet::Halt,
            _ => todo!(),
        }
    }

    fn exec_branch_ins(&mut self, ins: &BranchIns) -> ExecRet {
        let (z, n, c, v) = self.state.status.flags();
        let taken = match ins.op {
            BranchOpcode::Br => true,
            BranchOpcode::Bne => !z,
            BranchOpcode::Beq => z,
            BranchOpcode::Bmi => n,
            BranchOpcode::Bpl => !n,
            BranchOpcode::Bcs => c,
            BranchOpcode::Bcc => !c,
            BranchOpcode::Bvs => v,
            BranchOpcode::Bvc => !v,
            BranchOpcode::Blt => (n || v) && !(n && v),
            BranchOpcode::Bge => n == v,
            BranchOpcode::Ble => z || ((n || v) && !(n && v)),
            BranchOpcode::Bgt => !(z || ((n || v) && !(n && v))),
            BranchOpcode::Bhi => !c && !z,
            BranchOpcode::Blos => c || z,
        };


        if taken {
            let off = (ins.target.unwrap_offset() as i8) * 2;
            let pc = self.state.pc();
            let pc = pc.wrapping_add(off as i16 as u16);
            self.state.reg_write_word(Reg::PC,  pc);
            return ExecRet::Jmp;
        }

        ExecRet::Ok
    }


    fn exec_jmp_ins(&mut self, ins: &JmpIns) {
        assert_eq!(ins.op, JmpOpcode::Jmp);

        let dst = self.resolve(&ins.dst, Size::Word);
        assert!(!matches!(dst, ResolvedRegArg::Reg(_)));
        let new_pc = match dst {
            ResolvedRegArg::Mem(loc) => loc,
            dst => self.read_resolved_word(dst),
        };
        assert_eq!(new_pc & 0x1, 0);

        self.state.reg_write_word(Reg::PC,  new_pc);
    }

    fn push_word(&mut self, val: u16) {
        let sp = self.state.reg_read_word(Reg::SP) - 2;
        self.state.reg_write_word(Reg::SP, sp);
        self.mem_write_word(sp, val);
    }

    fn pop_word(&mut self) -> u16 {
        let sp = self.state.reg_read_word(Reg::SP);
        let val = self.mem_read_word(sp);
        self.state.reg_write_word(Reg::SP, sp + 2);
        val
    }

    fn exec_jsr_ins(&mut self, ins: &JsrIns) {
        assert_eq!(ins.op, JsrOpcode::Jsr);

        let dst = self.resolve(&ins.dst, Size::Word);
        assert!(!matches!(dst, ResolvedRegArg::Reg(_)));
        let new_pc = match dst {
            ResolvedRegArg::Mem(loc) => loc,
            dst => self.read_resolved_word(dst),
        };
        assert_eq!(new_pc & 0x1, 0);
        let old_val = self.state.reg_read_word(ins.reg);
        self.push_word(old_val);
        
        self.state.reg_write_word(ins.reg, self.state.pc());
        self.state.reg_write_word(Reg::PC, new_pc);
    }

    fn exec_rts_ins(&mut self, ins: &RtsIns) {
        assert_eq!(ins.op, RtsOpcode::Rts);
        let new_pc = self.state.reg_read_word(ins.reg);
        self.state.reg_write_word(Reg::PC, new_pc);
        
        let old_val = self.pop_word();
        self.state.reg_write_word(ins.reg, old_val);
    }

    fn exec_single_operand_ins(&mut self, ins: &SingleOperandIns) {
        let size = if ins.is_byte() { Size::Byte } else { Size::Word };
        let dst = self.resolve(&ins.dst, Size::Word);
        use SingleOperandOpcode::*;
        match ins.op {
            Swab => {
                let val = self.read_resolved_word(dst);
                let upper = val >> 8;
                let lower = val & ((1u16 << 8) - 1);
                let res = (lower << 8) | upper;

                self.write_resolved_word(dst, res);
                self.state.status.set_zero((res & 0xff) == 0);
                self.state.status.set_negative((res >> 7) & 0x1 == 1);
                self.state.status.set_carry(false);
                self.state.status.set_overflow(false);
            },
            Clr | ClrB => {
                self.write_resolved_narrow(dst, 0, size);
                self.state.status.set_zero(true);
                self.state.status.set_negative(false);
                self.state.status.set_carry(false);
                self.state.status.set_overflow(false);
            },
            Inc | IncB => {
                let val = self.read_resolved_widen(dst, size);
                let (res, _) = val.overflowing_add(1);

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero((res & size.mask()) == 0);
                self.state.status.set_negative(sign_bit(res, size) != 0);
                // Carry not affected
                self.state.status.set_overflow(val == (size.mask() >> 1));
                
            },
            Dec | DecB => {
                let val = self.read_resolved_widen(dst, size);
                let (res, _) = val.overflowing_sub(1);

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero(res & size.mask() == 0);
                self.state.status.set_negative(sign_bit(res, size) != 0);
                // Carry not affected
                self.state.status.set_overflow(val == size.smallest());
            },
            Neg | NegB => {
                let val = self.read_resolved_widen(dst, size);
                let res = (!val).wrapping_add(1);

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero(res & size.mask() == 0);
                self.state.status.set_negative(sign_bit(res, size) != 0);
                self.state.status.set_carry(res & size.mask() != 0);
                self.state.status.set_overflow(val == size.smallest());
            },
            Tst | TstB => {
                let val = self.read_resolved_widen(dst, size);
                let (res, _) = 0u32.overflowing_sub(val);

                self.state.status.set_zero(res == 0);
                self.state.status.set_negative(sign_bit(res, size) != 0);
                self.state.status.set_carry(false);
                self.state.status.set_overflow(false);
            },
            Com | ComB => {
                let val = self.read_resolved_widen(dst, size);
                let res = !val;

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero(res & size.mask() == 0);
                self.state.status.set_negative(sign_bit(res, size) != 0);
                self.state.status.set_carry(true);
                self.state.status.set_overflow(false);
            },
            Adc | AdcB => {
                let carry = self.state.status.get_carry();
                let val = self.read_resolved_widen(dst, size);
                let res = val.wrapping_add(carry as u32);

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero(res & size.mask() == 0);
                self.state.status.set_negative(sign_bit(res, size) != 0);
                self.state.status.set_carry(val == size.mask() && carry);
                self.state.status.set_overflow(val == size.largest() && carry);
            },
            Sbc | SbcB => {
                let carry = self.state.status.get_carry();
                let val = self.read_resolved_widen(dst, size);
                let res = val.wrapping_sub(carry as u32);

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero(res & size.mask() == 0);
                self.state.status.set_negative(sign_bit(res, size) != 0);
                self.state.status.set_carry(!((res & size.mask()) == 0 && carry));
                self.state.status.set_overflow(res == size.smallest());
            },
            Ror | RorB => {
                let val = self.read_resolved_widen(dst, size);
                let carry = self.state.status.get_carry() as u32;
                let new_carry = val & 0x1;
                let res = (val >> 1) | (carry << (size.bits() - 1));

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero((res & size.mask()) == 0);
                self.state.status.set_negative(sign_bit(res, size) != 0);
                self.state.status.set_carry(new_carry != 0);

                let n = self.state.status.get_negative() as u32;
                self.state.status.set_overflow((n ^ new_carry) != 0);
            },
            Rol | RolB => {
                let val = self.read_resolved_widen(dst, size);
                let carry = self.state.status.get_carry() as u32;
                let new_carry = (val >> (size.bits() - 1)) & 0x1;
                let res = (val << 1) | carry;

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero((res & size.mask()) == 0);
                self.state.status.set_negative(sign_bit(res, size) != 0);
                self.state.status.set_carry(new_carry != 0);

                let n = self.state.status.get_negative() as u32;
                self.state.status.set_overflow((n ^ new_carry) != 0);
            },
            Asr => {
                let val = self.read_resolved_word(dst);
                let new_carry = val & 0x1;
                let res = (val as i16) >> 1; // i16 for arithmetic shift

                self.write_resolved_word(dst, res as u16);
                self.state.status.set_zero(res == 0);
                self.state.status.set_negative(res >> 15 != 0);
                self.state.status.set_carry(new_carry != 0);

                let n = self.state.status.get_negative() as u16;
                self.state.status.set_overflow((n ^ new_carry) != 0);
            },
            Asl => {
                let val = self.read_resolved_word(dst);
                let res = val << 1;
                let new_carry = (val >> 15) & 0x1;

                self.write_resolved_word(dst, res);
                self.state.status.set_zero(res == 0);
                self.state.status.set_negative(res >> 15 != 0);
                self.state.status.set_carry(new_carry != 0);

                let n = self.state.status.get_negative() as u16;
                self.state.status.set_overflow((n ^ new_carry) != 0);
            },
            _ => panic!("{:?} not yet implemented", ins.op),
        }
    }

    fn exec_cc_ins(&mut self, ins: &CCIns) {
        let op = ins.op as u16;
        let bits = op & 0xf;
        let set = ((op >> 4) & 0x1) != 0;
        if set {
            self.state.status.set_flags(bits);
        } else {
            self.state.status.clear_flags(bits);
        }
    }

    fn exec(&mut self, ins: &Ins) -> ExecRet {
        match ins {
            Ins::DoubleOperand(ins) => { self.exec_double_operand_ins(ins); ExecRet::Ok },
            Ins::Branch(ins) => self.exec_branch_ins(ins),
            Ins::Jmp(ins) => { self.exec_jmp_ins(ins); ExecRet::Jmp },
            Ins::Jsr(ins) => { self.exec_jsr_ins(ins); ExecRet::Jmp },
            Ins::Rts(ins) => { self.exec_rts_ins(ins); ExecRet::Jmp },
            Ins::SingleOperand(ins) => { self.exec_single_operand_ins(ins); ExecRet::Ok },
            Ins::CC(ins) => { self.exec_cc_ins(ins); ExecRet::Ok },
            Ins::Misc(ins) => self.exec_misc_ins(ins),
            _ => todo!(),
        }
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use super::Emulator;
    use crate::emulator::DATA_START;
    use common::asm::Reg;
    use common::mem::as_byte_slice;

    #[test]
    fn halt() {
        let bin = &[
            0, // halt
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let mut emu = Emulator::new();
        emu.load_image(bin, DATA_START);
        emu.run_at(DATA_START);
        assert_eq!(emu.state.reg_read_word(Reg::PC), DATA_START + 2);
    }

    #[test]
    fn mov_reg_reg() {
        let bin = &[
            0o10001, // mov r0, r1
            0, // halt
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let val = 0xabcd;
        let mut emu = Emulator::new();
        emu.load_image(bin, DATA_START);
        emu.state.reg_write_word(Reg::R0, val);
        assert_eq!(emu.state.reg_read_word(Reg::R1), 0);
        emu.run_at(DATA_START);
        assert_eq!(emu.state.reg_read_word(Reg::R1), val);
    }

    #[test]
    fn mov_imm_reg() {
        let val = 0xabcd;
        let bin = &[
            0o12700, val, // mov #0xabcd, r0
            0,            // halt
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let mut emu = Emulator::new();
        emu.load_image(bin, DATA_START);
        assert_eq!(emu.state.reg_read_word(Reg::R0), 0);
        emu.run_at(DATA_START);
        assert_eq!(emu.state.reg_read_word(Reg::R0), val);
    }
    
    #[test]
    fn add() {
        let bin = &[
            0o5000,    // mov #0, r0
            0x65c0, 1, // add #1, r0
            0x0000     // halt
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let mut emu = Emulator::new();
        emu.load_image(bin, DATA_START);
        assert_eq!(emu.state.reg_read_word(Reg::R0), 0);
        emu.run_at(DATA_START);
        assert_eq!(emu.state.reg_read_word(Reg::R0), 1);
    }

    #[test]
    fn autoinc() {
        let arr = DATA_START + 18;
        let bin = &[
            0o12700, arr,   // mov  #arr, r0
            0o62720, 0o1,   // add  #1, (r0)+
            0o62720, 0o1,   // add  #1, (r0)+
            0o62720, 0o1,   // add  #1, (r0)+
            0o0,            // halt

        // arr:
            0o1, 0o2, 0o3   // .word 1 2 3
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let mut emu = Emulator::new();
        emu.load_image(bin, DATA_START);
        assert_eq!(emu.mem_read_word(arr), 1);
        assert_eq!(emu.mem_read_word(arr + 2), 2);
        assert_eq!(emu.mem_read_word(arr + 4), 3);
        emu.run_at(DATA_START);
        assert_eq!(emu.mem_read_word(arr), 2);
        assert_eq!(emu.mem_read_word(arr + 2), 3);
        assert_eq!(emu.mem_read_word(arr + 4), 4);
    }

    #[test]
    fn looop() {
        let bin = &[
            0o12700, 0,     // mov #0, r0
            0o12701, 10,    // mov #10, r1

            0o62700, 1,     // add #1, r0
            0o162701, 1,    // sub #1, r1
            0o1373,         // bne -10

            0               // halt
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let mut emu = Emulator::new();
        emu.load_image(bin, DATA_START);
        assert_eq!(emu.state.reg_read_word(Reg::R0), 0);
        emu.run_at(DATA_START);
        assert_eq!(emu.state.reg_read_word(Reg::R0), 10);
    }

    #[test]
    fn call() {
        let bin: &[u16] = &[
            0o12701, 0o0,   // mov #0, r1
            0o12702, 0o0,   // mov #0, r2
            0o407,          // br start

            0o12702, 0o2,   // mov #2, r2 ; shouldn't be executed

        // fun:
            0o12701, 0o1,   // mov #1, r1
            0o207,          // rts pc

            0o12702, 0o2,   // mov #2, r2 ; shouldn't be executed

        // start:
            0o4737, DATA_START + 0o16,   // jsr pc, fun
            0o0                          // halt
        ];
        let bin = unsafe { as_byte_slice(bin) };

        let mut emu = Emulator::new();
        emu.load_image(bin, DATA_START);
        emu.state.reg_write_word(Reg::SP, 2 * DATA_START);
        emu.run_at(DATA_START);
        assert_eq!(emu.state.reg_read_word(Reg::R1), 1);
        assert_eq!(emu.state.reg_read_word(Reg::R2), 0);
    }
}

fn not_and(src: u32, dst: u32) -> u32 {
    !src & dst
}

