
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};
use std::ops::{BitOr, BitAnd};

use num_traits::{FromPrimitive, ToPrimitive};    

use crate::common::asm::*;


pub trait MMIOHandler {
    fn read_byte(&mut self, addr: u16) -> u8;
    fn read_word(&mut self, addr: u16) -> u16;

    fn write_byte(&mut self, addr: u16, val: u8);
    fn write_word(&mut self, addr: u16, val: u16);
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
}


#[derive(Debug, Clone, Copy)]
enum ExecRet {
    Ok,
    Halt,
}

pub struct Emulator<'a> {
    mem: Vec<u8>,
    regs: [u16; NUM_REGS],
    mmio_handlers: HashMap<u16, MMIOHandlerRef<'a>>,
    status: Status,
}

impl<'a> Emulator<'a> {
    pub fn new(mem_size: u16) -> Emulator<'a> {
        assert!(mem_size <= MAX_MEM);
        Emulator {
            mem: vec![0; mem_size as usize],
            regs: [0; NUM_REGS],
            mmio_handlers: HashMap::new(),
            status: Status::new(),
        }
    }
    pub fn run(&mut self) {
        loop {
            let ins = self.decode();
            self.reg_write_word(Reg::PC, self.reg_read_word(Reg::PC) + 2);
            match self.exec(&ins) {
                ExecRet::Ok => continue,
                ExecRet::Halt => return,
            }
        }
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
        self.regs[reg.to_usize().unwrap()] = val;
    }

    pub fn reg_read_word(&self, reg: Reg) -> u16 {
        self.regs[reg.to_usize().unwrap()]
    }

    pub fn reg_read_byte(&self, reg: Reg) -> u8 {
        self.reg_read_word(reg) as u8
    }

    pub fn reg_write_byte(&mut self, reg: Reg, val: u8) {
        let mut old = self.reg_read_word(reg);
        old &= !(1u16 << 8) - 1;
        self.reg_write_word(reg, old | val as u16);
    }

    pub fn mem_read_byte(&mut self, addr: u16) -> u8 {
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                return handler.read_byte(addr);
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.mem[addr as usize]
        }
    }

    pub fn mem_write_byte(&mut self, addr: u16, val: u8) {
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                handler.write_byte(addr, val);
                return;
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.mem[addr as usize] = val;
        }
    }

    pub fn mem_read_word(&mut self, addr: u16) -> u16 {
        assert!(addr & 1 == 0);
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                return handler.read_word(addr);
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            (self.mem[addr as usize] as u16) | ((self.mem[(addr + 1) as usize] as u16) << 8)
        }
    }

    pub fn mem_write_word(&mut self, addr: u16, val: u16) {
        assert!(addr & 1 == 0);
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                handler.write_word(addr, val);
                return;
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.mem[addr as usize] = val as u8;
            self.mem[(addr + 1) as usize] = (val >> 8) as u8;
        }
    }

    fn write_resolved_word(&mut self, res: ResolvedRegArg, val: u16) {
        match res {
            ResolvedRegArg::Reg(r) => self.reg_write_word(r, val),
            ResolvedRegArg::Mem(addr) => self.mem_write_word(addr, val),
        }
    }

    fn write_resolved_byte(&mut self, res: ResolvedRegArg, val: u8) {
        match res {
            ResolvedRegArg::Reg(r) => self.reg_write_byte(r, val),
            ResolvedRegArg::Mem(addr) => self.mem_write_byte(addr, val),
        }
    }
    fn read_resolved_byte(&mut self, res: ResolvedRegArg) -> u8 {
        match res {
            ResolvedRegArg::Reg(r) => self.reg_read_byte(r),
            ResolvedRegArg::Mem(addr) => self.mem_read_byte(addr),
        }
    }
    fn read_resolved_word(&mut self, res: ResolvedRegArg) -> u16 {
        match res {
            ResolvedRegArg::Reg(r) => self.reg_read_word(r),
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
            AddrMode::AutoInc => self.exec_auto(arg.reg, true, size),
            AddrMode::AutoIncDef =>  self.exec_auto(arg.reg, true, size),
            AddrMode::AutoDec => self.exec_auto(arg.reg, false, size),
            AddrMode::AutoDecDef =>  self.exec_auto(arg.reg, false, size),
            AddrMode::Index => self.reg_read_word(arg.reg) + arg.extra.unwrap_imm(),
            AddrMode::IndexDef =>  self.mem_read_word(self.reg_read_word(arg.reg) + arg.extra.unwrap_imm()),
        };

        ResolvedRegArg::Mem(loc)
    }

    fn do_mov(&mut self, src: &RegArg, dst: &RegArg, size: Size) {
        let src = self.resolve(src, size);
        let val = self.read_resolved_word(src);
        let dst = self.resolve(dst, size);
        self.write_resolved_word(dst, val);
        self.status.set_zero(val == 0);
        self.status.set_negative(sign_bit(val as u32, size) != 0);
        self.status.set_overflow(false);
    }

    // TODO: combine these?
    fn do_bitwise(&mut self, src: &RegArg, op: fn(u32, u32) -> u32, dst: &RegArg, size: Size, discard: bool) {
        let src = self.resolve(&src, Size::Word);
        let src_val = self.read_resolved_widen(src, size);
        let dst = self.resolve(&dst, size);
        let dst_val = self.read_resolved_widen(dst, size);
        let res = op(src_val, dst_val);
        let res_sign = sign_bit(res, size);

        self.status.set_zero(res == 0);
        self.status.set_negative(res_sign != 0);
        // Carry not affected
        self.status.set_overflow(false);

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

        self.status.set_zero(res == 0);
        self.status.set_negative(res_sign != 0);
        self.status.set_carry(res >> size.bits() != 0);
        self.status.set_overflow(src_sign == dst_sign && dst_sign != res_sign);
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

        self.status.set_zero(res == 0);
        self.status.set_negative(res_sign != 0);
        self.status.set_carry(dst_val < src_val);
        self.status.set_overflow(src_sign != dst_sign && src_sign == res_sign);

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

    fn exec_branch_ins(&mut self, ins: &BranchIns) {
        let (z, n, c, v) = self.status.flags();
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
            let pc = self.reg_read_word(Reg::PC);
            let pc = if off < 0 {
                let off = TryInto::<u16>::try_into(-off).unwrap();
                pc - off
            } else {
                pc + TryInto::<u16>::try_into(off).unwrap()
            };
            self.reg_write_word(Reg::PC,  pc - 2); // -2 since pc was already incremented
        }
    }

    fn exec(&mut self, ins: &Ins) -> ExecRet {
        match ins {
            Ins::DoubleOperandIns(ins) => self.exec_double_operand_ins(ins),
            Ins::BranchIns(ins) => self.exec_branch_ins(ins),
            Ins::MiscIns(ins) => return self.exec_misc_ins(ins),
            _ => todo!(),
        }
        ExecRet::Ok
    }


    ///////////////////////////////////////////////////////////////////////////
    // Decode
    ///////////////////////////////////////////////////////////////////////////

    fn decode_reg_arg(&mut self, input: u16) -> RegArg {
        let reg = Reg::from_u16(input & Reg::MASK).unwrap();
        let mode = AddrMode::from_u16((input >> Reg::NUM_BITS) & AddrMode::MASK).unwrap();

        // The immediate is taken care of by pc-autoincrement
        RegArg{mode, reg, extra: Extra::None}
    }


    const DECODERS: [fn(&mut Emulator<'a>, u16) -> Result<Ins, ()>; 9] = [
        Self::decode_double_operand_ins,
        Self::decode_branch_ins,
        Self::decode_jmp_ins,
        Self::decode_jsr_ins,
        Self::decode_rts_ins,
        Self::decode_single_operand_ins,
        Self::decode_cc_ins,
        Self::decode_misc_ins,
        Self::decode_trap_ins,
    ];

    fn decode_double_operand_ins(&mut self, input: u16) -> Result<Ins, ()> {
        let op = DoubleOperandIns::decode_opcode(input)?;

        let src = self.decode_reg_arg(input >> RegArg::NUM_BITS);
        let dst = self.decode_reg_arg(input);

        Ok(Ins::DoubleOperandIns(DoubleOperandIns{op, src, dst}))
    }

    fn decode_branch_ins(&mut self, input: u16) -> Result<Ins, ()> {
        let op = BranchIns::decode_opcode(input)?;
        let offset = Target::Offset((input & BranchIns::OFFSET_MASK) as u8);
        Ok(Ins::BranchIns(BranchIns{op, target: offset}))
    }

    fn decode_jmp_ins(&mut self, input: u16) -> Result<Ins, ()> {
        let op = JmpIns::decode_opcode(input)?;
        let dst = self.decode_reg_arg(input);
        Ok(Ins::JmpIns(JmpIns{op, dst}))
    }

    fn decode_jsr_ins(&mut self, input: u16) -> Result<Ins, ()> {
        let op = JsrIns::decode_opcode(input)?;
        let dst = self.decode_reg_arg(input);
        let reg = Reg::from_u16((input >> RegArg::NUM_BITS) & Reg::MASK).unwrap();
        Ok(Ins::JsrIns(JsrIns{op, reg, dst}))
    }

    fn decode_rts_ins(&mut self, input: u16) -> Result<Ins, ()> {
        let op = RtsIns::decode_opcode(input)?;
        let reg = Reg::from_u16(input & Reg::MASK).unwrap();
        Ok(Ins::RtsIns(RtsIns{op, reg}))
    }

    fn decode_single_operand_ins(&mut self, input: u16) -> Result<Ins, ()> {
        let op = SingleOperandIns::decode_opcode(input)?;
        let dst = self.decode_reg_arg(input);
        Ok(Ins::SingleOperandIns(SingleOperandIns{op, dst}))
    }

    fn decode_cc_ins(&mut self, input: u16) -> Result<Ins, ()> {
        let op = CCIns::decode_opcode(input)?;
        Ok(Ins::CCIns(CCIns{op}))
    }

    fn decode_misc_ins(&mut self, input: u16) -> Result<Ins, ()> {
        let op = MiscIns::decode_opcode(input)?;
        Ok(Ins::MiscIns(MiscIns{op}))
    }

    fn decode_trap_ins(&mut self, input: u16) -> Result<Ins, ()> {
        let op = TrapIns::decode_opcode(input)?;
        let handler = Target::Offset((input & TrapIns::HANDLER_MASK) as u8);
        Ok(Ins::TrapIns(TrapIns{op, handler}))
    }

    fn decode(&mut self) -> Ins {
        let input = self.mem_read_word(self.reg_read_word(Reg::PC));
        for decoder in &Self::DECODERS {
            match decoder(self, input) {
                Ok(ins) => return ins,
                Err(()) => continue,
            }
        }

        panic!("Invalid instruction {:x}", input);
    }
}
