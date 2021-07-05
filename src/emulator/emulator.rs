
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::{BitOr, BitAnd};

use num_traits::ToPrimitive;    

use crate::common::asm::*;
use crate::common::decoder::decode;
use crate::common::mem::as_word_slice;


pub trait MMIOHandler {
    fn read_byte(&mut self, emu: &mut EmulatorData, addr: u16) -> u8;
    fn read_word(&mut self, emu: &mut EmulatorData, addr: u16) -> u16;

    fn write_byte(&mut self,  emu: &mut EmulatorData, addr: u16, val: u8);
    fn write_word(&mut self,  emu: &mut EmulatorData, addr: u16, val: u16);
}

type MMIOHandlerRef<'a>= &'a mut dyn MMIOHandler;


const NUM_REGS: usize = 8;
const MMIO_START: u16 = 0xe000;
pub const MAX_MEM: u16 = MMIO_START;

pub const DATA_START: u16 = 0x100;

pub struct Status(u16);

impl Status {
    const CARRY: usize = 0;
    const OVERFLOW: usize = 1;
    const ZERO: usize = 2;
    const NEGATIVE: usize = 3;
    const T: usize = 4;

    const PRIO: usize = 5;
    const PRIO_MASK: u16 = 0x7;


    fn new() -> Status {
        Status(0)
    }
    pub fn flags(&self) -> (bool, bool, bool, bool) {
        (self.get_zero(), self.get_negative(), self.get_carry(), self.get_overflow())
    }

    pub fn get_carry(&self) -> bool {
        ((self.0 >> Self::CARRY) & 0x1) != 0
    }

    pub fn set_carry(&mut self, val: bool) {
        self.0 &= !(1u16 << Self::CARRY);
        self.0 |= (val as u16) << Self::CARRY;
    }

    pub fn get_overflow(&self) -> bool {
        ((self.0 >> Self::OVERFLOW) & 0x1) != 0
    }

    pub fn set_overflow(&mut self, val: bool) {
        self.0 &= !(1u16 << Self::OVERFLOW);
        self.0 |= (val as u16) << Self::OVERFLOW;
    }

    pub fn get_zero(&self) -> bool {
        ((self.0 >> Self::ZERO) & 0x1) != 0
    }

    pub fn set_zero(&mut self, val: bool) {
        self.0 &= !(1u16 << Self::ZERO);
        self.0 |= (val as u16) << Self::ZERO;
    }

    pub fn get_negative(&self) -> bool {
        ((self.0 >> Self::ZERO) & 0x1) != 0
    }

    pub fn set_negative(&mut self, val: bool) {
        self.0 &= !(1u16 << Self::NEGATIVE);
        self.0 |= (val as u16) << Self::NEGATIVE;
    }

    pub fn get_t(&self) -> bool {
        ((self.0 >> Self::T) & 0x1) != 0
    }

    pub fn set_t(&mut self, val: bool) {
        self.0 &= !(1u16 << Self::T);
        self.0 |= (val as u16) << Self::T;
    }

    pub fn get_prio(&self) -> u8 {
        ((self.0 >> Self::PRIO) & Self::PRIO_MASK) as u8
    }

    pub fn set_prio(&mut self, val: u16) {
        assert!((val & !Self::PRIO_MASK) == 0);
        self.0 &= !(Self::PRIO_MASK << Self::PRIO);
        self.0 |= (val as u16) << Self::PRIO;
    }
}

#[derive(Debug, Clone, Copy)]
enum Size {
    Byte,
    Word,
}

impl Size {
    fn bytes(&self) -> u16 {
        match self {
            Size::Byte => 1,
            Size::Word => 2,
        }
    }

    fn bits(&self) -> u16 {
        self.bytes() * 8
    }
}

fn sign_bit(n: u32, size: Size) -> u32 {
    match size {
        Size::Word => n >> (15) & 0x1,
        Size::Byte => n >> (7) & 0x1,
    }
}

#[derive(Debug, Clone, Copy)]
enum ResolvedRegArg {
    Reg(Reg),
    Mem(u16),
    Imm(u16),
}

impl ResolvedRegArg {
    fn unwrap_mem(&self) -> u16 {
        match self {
            ResolvedRegArg::Mem(m) => *m,
            _ => panic!("ResolvedRegArg::unwrap_mem(): wasn't mem"),
        }
    }
}


#[derive(Debug, Clone, Copy)]
enum ExecRet {
    Ok,
    Jmp,
    Halt,
}


// This is separate so a mutable borrow can be passed to the MMIO handlers
pub struct EmulatorData {
    mem: Vec<u8>,
    regs: [u16; NUM_REGS],
    status: Status,
}

impl EmulatorData {
    pub fn mem_read_byte(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn mem_write_byte(&mut self, addr: u16, val: u8) {
        self.mem[addr as usize] = val;
    }

    pub fn mem_read_word(&mut self, addr: u16) -> u16 {
        assert!(addr & 1 == 0);
        (self.mem[addr as usize] as u16) | ((self.mem[(addr + 1) as usize] as u16) << 8)
    }

    pub fn mem_write_word(&mut self, addr: u16, val: u16) {
        assert!(addr & 1 == 0);
        self.mem[addr as usize] = val as u8;
        self.mem[(addr + 1) as usize] = (val >> 8) as u8;
    }
}


pub struct Emulator<'a> {
    data: EmulatorData,
    mmio_handlers: HashMap<u16, MMIOHandlerRef<'a>>,
}

impl<'a> Emulator<'a> {
    pub fn new(mem_size: u16) -> Emulator<'a> {
        assert!(mem_size <= MAX_MEM);
        Emulator {
            data: EmulatorData {
                mem: vec![0; mem_size as usize],
                regs: [0; NUM_REGS],
                status: Status::new(),
            },
            mmio_handlers: HashMap::new(),
        }
    }
    pub fn run(&mut self) {
        loop {
            let ins = self.decode();
            dbg!(self.pc(), &ins);
            let ins_size = ins.size();
            self.reg_write_word(Reg::PC, self.pc() + 2);
            match self.exec(&ins) {
                ExecRet::Ok => self.reg_write_word(Reg::PC, self.pc() + ins_size - 2),
                ExecRet::Jmp => (),
                ExecRet::Halt => return,
            }
        }
    }

    fn decode(&self) -> Ins {
        let pc = self.pc() as usize;
        let mem = &self.data.mem.as_slice()[pc..pc+6];
        let slice = unsafe { as_word_slice(mem) };
        decode(slice)
    }

    pub fn pc(&self) -> u16 {
        self.reg_read_word(Reg::PC)
    }

    pub fn run_at(&mut self, pc: u16) {
        self.reg_write_word(Reg::PC, pc);
        self.run();
    }

    pub fn load_image(&mut self, data: &[u8], start: u16) {
        let end = start + u16::try_from(data.len()).unwrap();
        for (byte, ptr) in data.iter().zip(start..end) {
            self.mem_write_byte(ptr, *byte);
        }
    }

    // Returns old handler for addr
    pub fn set_mmio_handler(&mut self, addr: u16, handler: MMIOHandlerRef<'a>) 
        -> Option<MMIOHandlerRef<'a>> {
        assert!(addr >= MMIO_START);
        self.mmio_handlers.insert(addr, handler)
    }

    ///////////////////////////////////////////////////////////////////////////

    pub fn reg_write_word(&mut self, reg: Reg, val: u16) {
        self.data.regs[reg.to_usize().unwrap()] = val;
    }

    pub fn reg_read_word(&self, reg: Reg) -> u16 {
        self.data.regs[reg.to_usize().unwrap()]
    }

    pub fn reg_read_byte(&self, reg: Reg) -> u8 {
        self.reg_read_word(reg) as u8
    }

    pub fn reg_write_byte(&mut self, reg: Reg, val: u8) {
        self.reg_write_word(reg, val as i8 as i16 as u16);
    }

    pub fn mem_read_byte(&mut self, addr: u16) -> u8 {
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                return handler.read_byte(&mut self.data, addr);
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.data.mem_read_byte(addr)
        }
    }

    pub fn mem_write_byte(&mut self, addr: u16, val: u8) {
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                handler.write_byte(&mut self.data, addr, val);
                return;
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.data.mem_write_byte(addr, val)
        }
    }

    pub fn mem_read_word(&mut self, addr: u16) -> u16 {
        assert!(addr & 1 == 0);
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                return handler.read_word(&mut self.data, addr);
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.data.mem_read_word(addr)
        }
    }

    pub fn mem_write_word(&mut self, addr: u16, val: u16) {
        assert!(addr & 1 == 0);
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                handler.write_word(&mut self.data, addr, val);
                return;
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.data.mem_write_word(addr, val)
        }
    }

    fn write_resolved_word(&mut self, res: ResolvedRegArg, val: u16) {
        match res {
            ResolvedRegArg::Reg(r) => self.reg_write_word(r, val),
            ResolvedRegArg::Mem(addr) => self.mem_write_word(addr, val),
            ResolvedRegArg::Imm(_) => panic!("Can't write to immediate"),
        }
    }

    fn write_resolved_byte(&mut self, res: ResolvedRegArg, val: u8) {
        match res {
            ResolvedRegArg::Reg(r) => self.reg_write_byte(r, val),
            ResolvedRegArg::Mem(addr) => self.mem_write_byte(addr, val),
            ResolvedRegArg::Imm(_) => panic!("Can't write to immediate"),
        }
    }
    fn read_resolved_byte(&mut self, res: ResolvedRegArg) -> u8 {
        match res {
            ResolvedRegArg::Reg(r) => self.reg_read_byte(r),
            ResolvedRegArg::Mem(addr) => self.mem_read_byte(addr),
            ResolvedRegArg::Imm(imm) => {
                assert_eq!(imm >> 8, 0);
                imm as u8
            },
        }
    }
    fn read_resolved_word(&mut self, res: ResolvedRegArg) -> u16 {
        match res {
            ResolvedRegArg::Reg(r) => self.reg_read_word(r),
            ResolvedRegArg::Mem(addr) => self.mem_read_word(addr),
            ResolvedRegArg::Imm(imm) => imm,
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
    fn exec_auto(&mut self, reg: Reg, inc: bool, size: Size) -> u16 {
        let mut val = self.reg_read_word(reg);
        if !inc { 
            val -= size.bytes();
        }
        let ret = val;
        if inc { 
            val += size.bytes();
        }
        self.reg_write_word(reg, val);
        ret
    }


    fn resolve(&mut self, arg: &RegArg, size: Size) -> ResolvedRegArg {
        let loc = match arg.mode {
            AddrMode::Gen => return ResolvedRegArg::Reg(arg.reg),
            AddrMode::Def => self.reg_read_word(arg.reg),
            AddrMode::AutoInc => {
                if arg.has_imm() {
                    return ResolvedRegArg::Imm(arg.extra.unwrap_imm());
                }
                self.exec_auto(arg.reg, true, size)
            }
            AddrMode::AutoIncDef => {
                if arg.has_imm() {
                    return ResolvedRegArg::Imm(arg.extra.unwrap_imm());
                }
                let addr = self.exec_auto(arg.reg, true, Size::Word);
                self.mem_read_word(addr)
            },
            AddrMode::AutoDec => {
                if arg.has_imm() {
                    return ResolvedRegArg::Imm(arg.extra.unwrap_imm());
                }
                self.exec_auto(arg.reg, false, size)
            }
            AddrMode::AutoDecDef => {
                if arg.has_imm() {
                    return ResolvedRegArg::Imm(arg.extra.unwrap_imm());
                }
                let addr = self.exec_auto(arg.reg, false, Size::Word);
                self.mem_read_word(addr)

            },
            // AddrMode::Index => self.reg_read_word(arg.reg).wrapping_add(arg.extra.unwrap_imm()),
            AddrMode::Index => {
                let reg_val = self.reg_read_word(arg.reg);
                let imm = arg.extra.unwrap_imm();
                let sum = reg_val.wrapping_add(imm);
                dbg!(reg_val, imm, sum);
                sum
            }
            AddrMode::IndexDef => self.mem_read_word(self.reg_read_word(arg.reg).wrapping_add(arg.extra.unwrap_imm())),
        };

        ResolvedRegArg::Mem(loc)
    }

    fn do_mov(&mut self, src: &RegArg, dst: &RegArg, size: Size) {
        let src = self.resolve(src, size);
        let val = self.read_resolved_word(src);
        let dst = self.resolve(dst, size);
        self.write_resolved_word(dst, val);
        self.data.status.set_zero(val == 0);
        self.data.status.set_negative(sign_bit(val as u32, size) != 0);
        self.data.status.set_overflow(false);
    }

    // TODO: combine these?
    fn do_bitwise(&mut self, src: &RegArg, op: fn(u32, u32) -> u32, dst: &RegArg, size: Size, discard: bool) {
        let src = self.resolve(&src, Size::Word);
        let src_val = self.read_resolved_widen(src, size);
        let dst = self.resolve(&dst, size);
        let dst_val = self.read_resolved_widen(dst, size);
        let res = op(src_val, dst_val);
        let res_sign = sign_bit(res, size);

        self.data.status.set_zero(res == 0);
        self.data.status.set_negative(res_sign != 0);
        // Carry not affected
        self.data.status.set_overflow(false);

        if !discard {
            self.write_resolved_narrow(dst, res, size);
        }
    }

    fn do_add(&mut self, src: &RegArg, dst: &RegArg, size: Size) {
        let src = self.resolve(src, size);
        let src_val = self.read_resolved_widen(src, size);
        let src_sign = sign_bit(src_val, size);
        let dst = self.resolve(dst, size);
        let dst_val = self.read_resolved_widen(dst, size);
        let dst_sign = sign_bit(dst_val, size);
        let res = src_val + dst_val;
        let res_sign = sign_bit(res, size);

        self.data.status.set_zero(res == 0);
        self.data.status.set_negative(res_sign != 0);
        self.data.status.set_carry(res >> size.bits() != 0);
        self.data.status.set_overflow(src_sign == dst_sign && dst_sign != res_sign);
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
        let res = dst_val - src_val;
        let res_sign = sign_bit(res, size);

        self.data.status.set_zero(res == 0);
        self.data.status.set_negative(res_sign != 0);
        self.data.status.set_carry(dst_val < src_val);
        self.data.status.set_overflow(src_sign != dst_sign && src_sign == res_sign);

        if !discard {
            self.write_resolved_narrow(dst, res, size);
        }
    }

    fn exec_double_operand_ins(&mut self, ins: &DoubleOperandIns) {
        match ins.op {
            DoubleOperandOpcode::Mov => self.do_mov(&ins.src, &ins.dst, Size::Word),
            DoubleOperandOpcode::Cmp => self.do_sub(&ins.dst, &ins.src, Size::Word, true),
            DoubleOperandOpcode::Bis => self.do_bitwise(&ins.src, u32::bitor, &ins.dst, Size::Word, false),
            DoubleOperandOpcode::Bic => self.do_bitwise(&ins.src, u32::bitand, &ins.dst, Size::Word, false),
            DoubleOperandOpcode::Bit => self.do_bitwise(&ins.src, u32::bitand, &ins.dst, Size::Word, true),

            DoubleOperandOpcode::Add => self.do_add(&ins.src, &ins.dst, Size::Word) ,

            DoubleOperandOpcode::MovB => self.do_mov(&ins.src, &ins.dst, Size::Byte),
            DoubleOperandOpcode::CmpB => self.do_sub(&ins.dst,  &ins.src, Size::Byte, true),
            DoubleOperandOpcode::BisB => self.do_bitwise(&ins.src, u32::bitor, &ins.dst, Size::Byte, false),
            DoubleOperandOpcode::BicB => self.do_bitwise(&ins.src, u32::bitand, &ins.dst, Size::Byte, false),
            DoubleOperandOpcode::BitB => self.do_bitwise(&ins.src, u32::bitand, &ins.dst, Size::Byte, true),

            DoubleOperandOpcode::Sub => self.do_sub(&ins.src, &ins.dst, Size::Word, false) ,
        }
    }

    fn exec_misc_ins(&mut self, ins: &MiscIns) -> ExecRet {
        match ins.op {
            MiscOpcode::Halt => return ExecRet::Halt,
            _ => todo!(),
        }
    }

    fn exec_branch_ins(&mut self, ins: &BranchIns) -> ExecRet {
        let (z, n, c, v) = self.data.status.flags();
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
            let pc = self.pc();
            let pc = pc.wrapping_add(off as i16 as u16);
            self.reg_write_word(Reg::PC,  pc);
            return ExecRet::Jmp;
        }

        return ExecRet::Ok;
    }


    fn exec_jmp_ins(&mut self, ins: &JmpIns) {
        assert_eq!(ins.op, JmpOpcode::Jmp);
        let new_pc = self.resolve(&ins.dst, Size::Word).unwrap_mem();
        assert_eq!(new_pc & 0x1, 0);
        self.reg_write_word(Reg::PC,  new_pc);
    }

    fn push_word(&mut self, val: u16) {
        let sp = self.reg_read_word(Reg::SP) - 2;
        self.reg_write_word(Reg::SP, sp);
        self.mem_write_word(sp, val);
    }

    fn pop_word(&mut self) -> u16 {
        let sp = self.reg_read_word(Reg::SP);
        let val = self.mem_read_word(sp);
        self.reg_write_word(Reg::SP, sp + 2);
        val
    }

    fn exec_jsr_ins(&mut self, ins: &JsrIns) {
        assert_eq!(ins.op, JsrOpcode::Jsr);

        let new_pc = self.resolve(&ins.dst, Size::Word).unwrap_mem();
        assert_eq!(new_pc & 0x1, 0);

        if ins.reg == Reg::PC {
            // This is a hack, since its not clear when the extra pc increment for the index is
            // supposed to happen
            let ret_addr = self.pc() + ins.num_imm() * 2;
            self.reg_write_word(ins.reg, ret_addr);
        }
        let old_val = self.reg_read_word(ins.reg);
        self.push_word(old_val);
        
        self.reg_write_word(ins.reg, self.pc());
        self.reg_write_word(Reg::PC, new_pc);
    }

    fn exec_rts_ins(&mut self, ins: &RtsIns) {
        assert_eq!(ins.op, RtsOpcode::Rts);
        let new_pc = self.reg_read_word(ins.reg);
        self.reg_write_word(Reg::PC, new_pc);
        
        let old_val = self.pop_word();
        self.reg_write_word(ins.reg, old_val);
    }

    fn exec_single_operand_ins(&mut self, ins: &SingleOperandIns) {
        let dst = self.resolve(&ins.dst, Size::Word);
        match ins.op {
            SingleOperandOpcode::Swab => {
                let val = self.read_resolved_word(dst);
                let upper = val >> 7;
                let lower = val & ((1u16 << 7) - 1);
                let res = (lower << 7) | upper;

                self.write_resolved_word(dst, res);
                self.data.status.set_zero(upper == 0);
                self.data.status.set_negative((res >> 7) & 0x1 == 0);
                self.data.status.set_carry(false);
                self.data.status.set_overflow(false);
            },
            SingleOperandOpcode::Clr => {
                self.write_resolved_word(dst, 0);
                self.data.status.set_zero(true);
                self.data.status.set_negative(false);
                self.data.status.set_carry(false);
                self.data.status.set_overflow(false);
            },
            SingleOperandOpcode::Inc => {
                let val = self.read_resolved_word(dst);
                let (res, _) = val.overflowing_add(1);

                self.write_resolved_word(dst, res);
                self.data.status.set_zero(res == 0);
                self.data.status.set_negative(res >> 15 != 0);
                // Carry not affected
                self.data.status.set_overflow(val == 0o77777);
                
            },
            SingleOperandOpcode::Dec => {
                let val = self.read_resolved_word(dst);
                let (res, _) = val.overflowing_sub(1);

                self.write_resolved_word(dst, res);
                self.data.status.set_zero(res == 0);
                self.data.status.set_negative(res >> 15 != 0);
                // Carry not affected
                self.data.status.set_overflow(val == 0o100000);
            },
            SingleOperandOpcode::Neg => {
                let val = self.read_resolved_word(dst);
                let res = !val + 1;

                self.write_resolved_word(dst, res);
                self.data.status.set_zero(res == 0);
                self.data.status.set_negative(res >> 15 != 0);
                self.data.status.set_carry(res != 0);
                self.data.status.set_overflow(res == 0o100000);
            },
            SingleOperandOpcode::Tst => {
                let val = self.read_resolved_word(dst);
                let (res, _) = 0u16.overflowing_sub(val);

                self.data.status.set_zero(res == 0);
                self.data.status.set_negative(res >> 15 != 0);
                self.data.status.set_carry(false);
                self.data.status.set_overflow(false);
            },
            SingleOperandOpcode::Com => {
                let val = self.read_resolved_word(dst);
                let res = !val;

                self.write_resolved_word(dst, res);
                self.data.status.set_zero(res == 0);
                self.data.status.set_negative(res >> 15 != 0);
                self.data.status.set_carry(true);
                self.data.status.set_overflow(false);
            },
            SingleOperandOpcode::Adc => {
                let carry = self.data.status.get_carry();
                let val = self.read_resolved_word(dst);
                let res = val + carry as u16;

                self.write_resolved_word(dst, res);
                self.data.status.set_zero(res == 0);
                self.data.status.set_negative(res >> 15 != 0);
                self.data.status.set_carry(val == 0o177777 && carry);
                self.data.status.set_overflow(val == 0o077777 && carry);
            },
            SingleOperandOpcode::Sbc => {
                let carry = self.data.status.get_carry();
                let val = self.read_resolved_word(dst);
                let res = val - carry as u16;

                self.write_resolved_word(dst, res);
                self.data.status.set_zero(res == 0);
                self.data.status.set_negative(res >> 15 != 0);
                self.data.status.set_carry(val == 0 && carry);
                self.data.status.set_overflow(val == 0o100000);
            },
            SingleOperandOpcode::Ror => {
                let val = self.read_resolved_word(dst);
                let carry = self.data.status.get_carry() as u16;
                let new_carry = val & 0x1;
                let res = (val >> 1) | (carry << 15);

                self.write_resolved_word(dst, res);
                self.data.status.set_zero(res == 0);
                self.data.status.set_negative(res >> 15 != 0);
                self.data.status.set_carry(new_carry != 0);

                let n = self.data.status.get_negative() as u16;
                self.data.status.set_overflow((n ^ new_carry) != 0);
            },
            SingleOperandOpcode::Rol => {
                let val = self.read_resolved_word(dst);
                let carry = self.data.status.get_carry() as u16;
                let new_carry = (val >> 15) & 0x1;
                let res = (val << 1) | carry;

                self.write_resolved_word(dst, res);
                self.data.status.set_zero(res == 0);
                self.data.status.set_negative(res >> 15 != 0);
                self.data.status.set_carry(new_carry != 0);

                let n = self.data.status.get_negative() as u16;
                self.data.status.set_overflow((n ^ new_carry) != 0);
            },
            SingleOperandOpcode::Asr => {
                let val = self.read_resolved_word(dst);
                let new_carry = val & 0x1;
                let res = (val as i16) >> 1; // i16 for arithmetic shift

                self.write_resolved_word(dst, res as u16);
                self.data.status.set_zero(res == 0);
                self.data.status.set_negative(res >> 15 != 0);
                self.data.status.set_carry(new_carry != 0);

                let n = self.data.status.get_negative() as u16;
                self.data.status.set_overflow((n ^ new_carry) != 0);
            },
            SingleOperandOpcode::Asl => {
                let val = self.read_resolved_word(dst);
                let res = val << 1;
                let new_carry = (val >> 15) & 0x1;

                self.write_resolved_word(dst, res);
                self.data.status.set_zero(res == 0);
                self.data.status.set_negative(res >> 15 != 0);
                self.data.status.set_carry(new_carry != 0);

                let n = self.data.status.get_negative() as u16;
                self.data.status.set_overflow((n ^ new_carry) != 0);
            },
            _ => panic!("{:?} not yet implemented", ins.op),
        }
    }

    fn exec(&mut self, ins: &Ins) -> ExecRet {
        match ins {
            Ins::DoubleOperandIns(ins) => { self.exec_double_operand_ins(ins); ExecRet::Ok },
            Ins::BranchIns(ins) => self.exec_branch_ins(ins),
            Ins::JmpIns(ins) => { self.exec_jmp_ins(ins); ExecRet::Jmp },
            Ins::JsrIns(ins) => { self.exec_jsr_ins(ins); ExecRet::Jmp },
            Ins::RtsIns(ins) => { self.exec_rts_ins(ins); ExecRet::Jmp },
            Ins::SingleOperandIns(ins) => { self.exec_single_operand_ins(ins); ExecRet::Ok },
            Ins::MiscIns(ins) => self.exec_misc_ins(ins),
            _ => todo!(),
        }
    }


}
