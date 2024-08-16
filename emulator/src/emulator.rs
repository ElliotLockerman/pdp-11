
use common::asm::*;
use common::decoder::decode;
use common::constants::*;
use crate::MMIOHandler;
use crate::io::Interrupt;
use crate::io::status_access::StatusAccess;
use crate::EmulatorState;
use crate::Status;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::{BitOr, BitAnd};
use std::sync::{Arc, Mutex};
use std::cmp::Ordering;

use log::{debug, trace};
use num_traits::{ToPrimitive, FromPrimitive};

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

    fn smallest_signed(self) -> u32 {
        0x1 << (self.bits() - 1)
    }

    fn largest_signed(self) -> u32 {
        self.smallest_signed().wrapping_sub(1)
    }

    fn sign_bit(self, val: u32) -> u32 {
        match self {
            Size::Word => (val >> 15) & 0x1,
            Size::Byte => (val >> 7) & 0x1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ResolvedOperand {
    Reg(Reg),
    Mem(u16),
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecRet {
    Ok,
    Halt,
    Wait,
}


pub struct Emulator {
    state: EmulatorState,
    mmio_handlers: HashMap<u16, Arc<Mutex<dyn MMIOHandler>>>,
    waiting: bool,
}

impl Emulator {
    pub fn new() -> Emulator {
        let mut emu = Emulator {
            state: EmulatorState::new(),
            mmio_handlers: HashMap::new(),
            waiting: false,
        };
        emu.set_mmio_handler(StatusAccess::default());
        emu
    }

    // Run until a halt.
    pub fn run(&mut self) {
        while self.run_ins() != ExecRet::Halt {}
    }

    // Run a single instruction, letting each device get a time slice and potentially
    // generating an interrupt.
    pub fn run_ins(&mut self) -> ExecRet {
        // TODO: better timing model
        self.state.inc_ins();

        if let Some((dev, inter)) = self.tick_devices() {
            if inter.prio > self.get_state().get_status().get_prio() {
                self.waiting = false;
                dev.lock().unwrap().interrupt_accepted();
                self.interrupt(inter.vector);
            }
        }
        
        if self.waiting {
            return ExecRet::Wait;
        }

        let ins = self.decode();
        debug!("PC: 0o{:o}: {}", self.state.pc(), ins.display_with_pc(self.state.pc()));
        self.state.reg_write_word(Reg::PC, self.state.pc() + 2);

        if matches!(ins, Ins::Misc(MiscIns{op: MiscOpcode::Wait})) {
            self.waiting = true;
            return ExecRet::Wait;
        }

        self.exec(&ins)
    }

    // Continue after halt.
    pub fn cont(&mut self) {
        self.run();
    }

    fn tick_devices(&mut self) -> Option<(Arc<Mutex<dyn MMIOHandler>>, Interrupt)> {
        let mut interrupt: Option<(Arc<Mutex<dyn MMIOHandler>>, Interrupt)>  = None;
        for dev in self.mmio_handlers.values_mut() {
            if let Some(inter) = dev.lock().unwrap().tick(&mut self.state) {
                if let Some(max) = &interrupt {
                    if inter.prio > max.1.prio {
                        interrupt = Some((dev.clone(), inter))
                    }
                } else {
                    interrupt = Some((dev.clone(), inter))
                }
            }
        }
        interrupt
    }

    fn decode(&self) -> Ins {
        let next_ins = self.state.next_ins();
        let Some(ins) = decode(next_ins) else {
            panic!("Invalid instruction 0{:o}", next_ins[0]);
        };
        ins
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

    pub fn set_mmio_handler_for<M, I>(&mut self, handler: M, addrs: I) 
    where 
        M: MMIOHandler + 'static,
        I: IntoIterator<Item = u16> {

        let handler = Arc::new(Mutex::new(handler));
        for addr in addrs.into_iter() {
            self.register_handler(handler.clone(), addr);
        }
    }

    pub fn set_mmio_handler(&mut self, handler: impl MMIOHandler + 'static) {
        let handler = Arc::new(Mutex::new(handler));
        for addr in handler.lock().unwrap().default_addrs() {
            self.register_handler(handler.clone(), *addr);
        }
    }

    fn register_handler(&mut self, handler: Arc<Mutex<dyn MMIOHandler>>, addr: u16) {
        assert!(addr >= MMIO_START);
        assert!(addr & 0x1 == 0, "MMIOHandler addr {addr:o} not aligned");
        let prev = self.mmio_handlers.insert(addr, handler);
        assert!(prev.is_none(), "Duplicate MMIOHandler for {addr:o}");
    }


    ///////////////////////////////////////////////////////////////////////////


    pub fn mem_read_byte(&mut self, addr: u16) -> u8 {
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                return handler.lock().unwrap().read_byte(&mut self.state, addr);
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.state.mem_read_byte(addr)
        }
    }

    pub fn mem_write_byte(&mut self, addr: u16, val: u8) {
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                handler.lock().unwrap().write_byte(&mut self.state, addr, val);
                return;
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.state.mem_write_byte(addr, val)
        }
    }

    pub fn mem_read_word(&mut self, addr: u16) -> u16 {
        assert!(addr & 1 == 0, "Word read of 0o{addr:o} not aligned");
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                return handler.lock().unwrap().read_word(&mut self.state, addr);
            }
            panic!("Invalid MMIO register {}", addr);
        } else {
            self.state.mem_read_word(addr)
        }
    }

    pub fn mem_write_word(&mut self, addr: u16, val: u16) {
        assert!(addr & 1 == 0, "Word write of 0o{addr:o} not aligned");
        if addr >= MMIO_START {
            if let Some(handler) = self.mmio_handlers.get_mut(&addr) {
                handler.lock().unwrap().write_word(&mut self.state, addr, val);
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

    fn write_resolved_word(&mut self, res: ResolvedOperand, val: u16) {
        match res {
            ResolvedOperand::Reg(r) => self.state.reg_write_word(r, val),
            ResolvedOperand::Mem(addr) => self.mem_write_word(addr, val),
        }
    }

    fn write_resolved_byte(&mut self, res: ResolvedOperand, val: u8) {
        match res {
            ResolvedOperand::Reg(r) => self.state.reg_write_byte(r, val),
            ResolvedOperand::Mem(addr) => self.mem_write_byte(addr, val),
        }
    }
    fn read_resolved_byte(&mut self, res: ResolvedOperand) -> u8 {
        match res {
            ResolvedOperand::Reg(r) => self.state.reg_read_byte(r),
            ResolvedOperand::Mem(addr) => self.mem_read_byte(addr),
        }
    }
    fn read_resolved_word(&mut self, res: ResolvedOperand) -> u16 {
        match res {
            ResolvedOperand::Reg(r) => self.state.reg_read_word(r),
            ResolvedOperand::Mem(addr) => self.mem_read_word(addr),
        }
    }

    fn read_resolved_widen(&mut self, res: ResolvedOperand, size: Size) -> u32 {
        match size {
            Size::Word => self.read_resolved_word(res) as u32,
            Size::Byte => self.read_resolved_byte(res) as u32,
        }
    }

    fn write_resolved_narrow(&mut self, res: ResolvedOperand, val: u32, size: Size) {
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
            val = val.wrapping_sub(size.bytes());
        }
        let ret = val;
        if inc { 
            val = val.wrapping_add(size.bytes());
        }
        self.state.reg_write_word(reg, val);
        ret
    }

    #[inline]
    fn debug_check_extra_val(arg: &Operand, val: u16) {
        if arg.needs_extra() {
            debug_assert_eq!(arg.extra.unwrap_val(), val);
        }
    }

    #[inline]
    fn debug_check_extra_addr(&self, arg: &Operand, addr: u16) {
        if arg.needs_extra() {
            debug_assert_eq!(arg.extra.unwrap_val(), self.get_state().mem_read_word(addr));
        }
    }

    // Convert an operand to a register or a memory location that can be read
    // or written. This is a separate function from read and write, because some
    // regs/addrs may get both a read and write in one instruction, but resolving
    // the operand is side-effecting.
    // There's a little bit of overlap in functionality between this function
    // and the decoding done by common::decode::decoder. The emulator version
    // is more similar to the "proper" model, what with it both having and relying
    // on side effects on the emulator state, but this is not convenient for other
    // tools, e.g., disassembers. the debug_check_extra_* functions make sure
    // they are equivalent (in debug mode).
    fn resolve(&mut self, arg: &Operand, size: Size) -> ResolvedOperand {
        let loc = match arg.mode {
            AddrMode::Gen => return ResolvedOperand::Reg(arg.reg),
            AddrMode::Def => self.state.reg_read_word(arg.reg),
            AddrMode::AutoInc => {
                let addr = self.exec_auto(arg.reg, true, size);
                self.debug_check_extra_addr(arg, addr);
                addr
            }
            AddrMode::AutoIncDef => {
                let addr = self.exec_auto(arg.reg, true, Size::Word);
                self.debug_check_extra_addr(arg, addr);
                self.mem_read_word(addr)
            },
            AddrMode::AutoDec => {
                let addr = self.exec_auto(arg.reg, false, size);
                self.debug_check_extra_addr(arg, addr);
                addr
            }
            AddrMode::AutoDecDef => {
                let addr = self.exec_auto(arg.reg, false, Size::Word);
                self.debug_check_extra_addr(arg, addr);
                self.mem_read_word(addr)
            },
            AddrMode::Index => {
                let reg_val = self.state.reg_read_word(arg.reg);
                let imm_addr = self.exec_auto(Reg::PC, true, Size::Word);
                let imm = self.mem_read_word(imm_addr);
                Self::debug_check_extra_val(arg, imm);
                reg_val.wrapping_add(imm)
            },
            AddrMode::IndexDef => {
                let reg_val = self.state.reg_read_word(arg.reg);
                let imm_addr = self.exec_auto(Reg::PC, true, Size::Word);
                let imm = self.mem_read_word(imm_addr);
                Self::debug_check_extra_val(arg, imm);
                self.mem_read_word(reg_val.wrapping_add(imm))
            },
        };


        ResolvedOperand::Mem(loc)
    }

    fn do_mov(&mut self, src: &Operand, dst: &Operand, size: Size) {
        let src = self.resolve(src, size);
        let val = self.read_resolved_widen(src, size);
        let dst = self.resolve(dst, size);

        if size == Size::Byte {
            if matches!(dst, ResolvedOperand::Reg(_)) {
                let val = val as u8 as i8 as i16 as u16;
                self.write_resolved_word(dst, val);
            } else {
                self.write_resolved_narrow(dst, val, size);
            }
        } else {
            self.write_resolved_word(dst, val as u16);
        }
        self.state.status.set_zero(val == 0);
        self.state.status.set_negative(size.sign_bit(val) != 0);
        self.state.status.set_overflow(false);
    }

    // TODO: combine these?
    fn do_bitwise(&mut self, src: &Operand, op: fn(u32, u32) -> u32, dst: &Operand, size: Size, discard: bool) {
        let src = self.resolve(src, size);
        let src_val = self.read_resolved_widen(src, size);
        let dst = self.resolve(dst, size);
        let dst_val = self.read_resolved_widen(dst, size);
        let res = op(src_val, dst_val);
        let res_sign = size.sign_bit(res);

        self.state.status.set_zero(res == 0);
        self.state.status.set_negative(res_sign != 0);
        // Carry not affected
        self.state.status.set_overflow(false);

        if !discard {
            self.write_resolved_narrow(dst, res, size);
        }
    }

    fn do_add(&mut self, src: &Operand, dst: &Operand, size: Size) {
        assert!(size == Size::Word);
        let src = self.resolve(src, size);
        let src_val = self.read_resolved_widen(src, size);
        let src_sign = size.sign_bit(src_val);
        let dst = self.resolve(dst, size);
        let dst_val = self.read_resolved_widen(dst, size);
        let dst_sign = size.sign_bit(dst_val);
        let res = src_val + dst_val;
        let res_sign = size.sign_bit(res);

        self.state.status.set_zero((res & size.mask()) == 0);
        self.state.status.set_negative(res_sign != 0);
        self.state.status.set_carry(res >> size.bits() != 0);
        self.state.status.set_overflow(src_sign == dst_sign && dst_sign != res_sign);
        self.write_resolved_narrow(dst, res, size);
    }

    fn do_sub(&mut self, src: &Operand, dst: &Operand, size: Size) {
        let src = self.resolve(src, size);
        let src_val = self.read_resolved_widen(src, size);
        let src_sign = size.sign_bit(src_val);
        let dst = self.resolve(dst, size);
        let dst_val = self.read_resolved_widen(dst, size);
        let dst_sign = size.sign_bit(dst_val);
        let res = dst_val.wrapping_add((!src_val).wrapping_add(1) & size.mask());
        let res_sign = size.sign_bit(res);

        self.state.status.set_zero((res & size.mask()) == 0);
        self.state.status.set_negative(res_sign != 0);
        self.state.status.set_carry(dst_val < src_val);
        self.state.status.set_overflow(src_sign != dst_sign && src_sign == res_sign);

        self.write_resolved_narrow(dst, res, size);
    }


    // NB: args are swapped compared to sub!
    fn do_cmp(&mut self, src: &Operand, dst: &Operand, size: Size) {
        let src = self.resolve(src, size);
        let src_val = self.read_resolved_widen(src, size);
        let src_sign = size.sign_bit(src_val);
        let dst = self.resolve(dst, size);
        let dst_val = self.read_resolved_widen(dst, size);
        let dst_sign = size.sign_bit(dst_val);
        let res = src_val.wrapping_add((!dst_val).wrapping_add(1) & size.mask());
        let res_sign = size.sign_bit(res);

        self.state.status.set_zero((res & size.mask()) == 0);
        self.state.status.set_negative(res_sign != 0);
        self.state.status.set_carry(src_val < dst_val);
        self.state.status.set_overflow(src_sign != dst_sign && dst_sign == res_sign);
    }

    fn exec_double_operand_ins(&mut self, ins: &DoubleOperandIns) {
        use DoubleOperandOpcode::*;
        match ins.op {
            Mov => self.do_mov(&ins.src, &ins.dst, Size::Word),
            Cmp => self.do_cmp(&ins.src, &ins.dst, Size::Word),
            Bis => self.do_bitwise(&ins.src, u32::bitor, &ins.dst, Size::Word, false),
            Bic => self.do_bitwise(&ins.src, not_and, &ins.dst, Size::Word, false),
            Bit => self.do_bitwise(&ins.src, u32::bitand, &ins.dst, Size::Word, true),

            Add => self.do_add(&ins.src, &ins.dst, Size::Word) ,

            MovB => self.do_mov(&ins.src, &ins.dst, Size::Byte),
            CmpB => self.do_cmp(&ins.src,  &ins.dst, Size::Byte),
            BisB => self.do_bitwise(&ins.src, u32::bitor, &ins.dst, Size::Byte, false),
            BicB => self.do_bitwise(&ins.src, not_and, &ins.dst, Size::Byte, false),
            BitB => self.do_bitwise(&ins.src, u32::bitand, &ins.dst, Size::Byte, true),

            Sub => self.do_sub(&ins.src, &ins.dst, Size::Word) ,
        }
    }

    fn exec_misc_ins(&mut self, ins: &MiscIns) -> ExecRet {
        match ins.op {
            MiscOpcode::Halt => { return ExecRet::Halt; },
            MiscOpcode::Rti => self.exec_rti_ins(),
            _ => panic!(
                "Instruction {ins:?} (0o{:o}) at pc 0o{:o} not yet implemented",
                ins.op as u16,
                self.state.reg_read_word(Reg::PC)
            ),
        }
        ExecRet::Ok
    }

    fn exec_branch_ins(&mut self, ins: &BranchIns) {
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
        }
    }


    fn exec_jmp_ins(&mut self, ins: &JmpIns) {
        assert_eq!(ins.op, JmpOpcode::Jmp);

        let dst = self.resolve(&ins.dst, Size::Word);
        assert!(!matches!(dst, ResolvedOperand::Reg(_)));
        let new_pc = match dst {
            ResolvedOperand::Mem(loc) => loc,
            dst => self.read_resolved_word(dst),
        };
        assert_eq!(new_pc & 0x1, 0);

        trace!("PC: 0o{:o}: JMP to 0o{new_pc:o}", self.state.pc());
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
        assert!(!matches!(dst, ResolvedOperand::Reg(_)));
        let new_pc = match dst {
            ResolvedOperand::Mem(loc) => loc,
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
                self.state.status.set_negative(size.sign_bit(res) != 0);
                // Carry not affected
                self.state.status.set_overflow(val == (size.mask() >> 1));
                
            },
            Dec | DecB => {
                let val = self.read_resolved_widen(dst, size);
                let (res, _) = val.overflowing_sub(1);

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero(res & size.mask() == 0);
                self.state.status.set_negative(size.sign_bit(res) != 0);
                // Carry not affected
                self.state.status.set_overflow(val == size.smallest_signed());
            },
            Neg | NegB => {
                let val = self.read_resolved_widen(dst, size);
                let res = (!val).wrapping_add(1);

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero(res & size.mask() == 0);
                self.state.status.set_negative(size.sign_bit(res) != 0);
                self.state.status.set_carry(res & size.mask() != 0);
                self.state.status.set_overflow(val == size.smallest_signed());
            },
            Tst | TstB => {
                let val = self.read_resolved_widen(dst, size);
                let (res, _) = 0u32.overflowing_sub(val);

                self.state.status.set_zero(res == 0);
                self.state.status.set_negative(size.sign_bit(res) != 0);
                self.state.status.set_carry(false);
                self.state.status.set_overflow(false);
            },
            Com | ComB => {
                let val = self.read_resolved_widen(dst, size);
                let res = !val;

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero(res & size.mask() == 0);
                self.state.status.set_negative(size.sign_bit(res) != 0);
                self.state.status.set_carry(true);
                self.state.status.set_overflow(false);
            },
            Adc | AdcB => {
                let carry = self.state.status.get_carry();
                let val = self.read_resolved_widen(dst, size);
                let res = val.wrapping_add(carry as u32);

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero(res & size.mask() == 0);
                self.state.status.set_negative(size.sign_bit(res) != 0);
                self.state.status.set_carry(val == size.mask() && carry);
                self.state.status.set_overflow(val == size.largest_signed() && carry);
            },
            Sbc | SbcB => {
                let carry = self.state.status.get_carry();
                let val = self.read_resolved_widen(dst, size);
                let res = val.wrapping_sub(carry as u32);

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero(res & size.mask() == 0);
                self.state.status.set_negative(size.sign_bit(res) != 0);
                self.state.status.set_carry(!((res & size.mask()) == 0 && carry));
                self.state.status.set_overflow(res == size.smallest_signed());
            },
            Ror | RorB => {
                let val = self.read_resolved_widen(dst, size);
                let carry = self.state.status.get_carry() as u32;
                let new_carry = val & 0x1;
                let res = (val >> 1) | (carry << (size.bits() - 1));

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero((res & size.mask()) == 0);
                self.state.status.set_negative(size.sign_bit(res) != 0);
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
                self.state.status.set_negative(size.sign_bit(res) != 0);
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
            AsrB => {
                let val = self.read_resolved_byte(dst);
                let new_carry = val & 0x1;
                let res = (val as i8) >> 1; // i16 for arithmetic shift

                self.write_resolved_byte(dst, res as u8);
                self.state.status.set_zero(res == 0);
                self.state.status.set_negative(res >> 7 != 0);
                self.state.status.set_carry(new_carry != 0);

                let n = self.state.status.get_negative() as u8;
                self.state.status.set_overflow((n ^ new_carry) != 0);
            },
            Asl | AslB => {
                let val = self.read_resolved_widen(dst, size);
                let res = val << 1;
                let new_carry = size.sign_bit(val);

                self.write_resolved_narrow(dst, res, size);
                self.state.status.set_zero(res & size.mask() == 0);
                self.state.status.set_negative(size.sign_bit(res) != 0);
                self.state.status.set_carry(new_carry != 0);

                let n = self.state.status.get_negative() as u32;
                self.state.status.set_overflow((n ^ new_carry) != 0);
            },
        }
    }

    fn exec_eis_ins(&mut self, ins: &EisIns) {
        let operand = self.resolve(&ins.operand, Size::Word);
        let operand_val = self.read_resolved_word(operand);

        use EisOpcode::*;
        if ins.op == Xor {
            let dst = operand;
            let dst_val = operand_val;

            let val = self.state.reg_read_word(ins.reg);
            let res = dst_val ^ val;
            self.write_resolved_word(dst, res);

            self.state.status.set_negative(Size::Word.sign_bit(res as u32) != 0);
            self.state.status.set_zero(res == 0);
            self.state.status.set_overflow(false);
            // Carry unaffected
            return;
        }

        let src_val = operand_val;
        let reg_val = self.state.reg_read_word(ins.reg);
        match ins.op {
            Mul => {
                let src_val = src_val as i16 as i32;
                let reg_val = reg_val as i16 as i32;
                let res = src_val * reg_val;

                self.state.status.set_negative(res < 0);
                self.state.status.set_zero(res == 0);
                self.state.status.set_overflow(false);
                self.state.status.set_carry(i16::try_from(res).is_err());

                self.state.reg_write_word(ins.reg, res as u16);
                let reg_num = ins.reg.to_u16().unwrap();
                if reg_num & 0x1 == 0 {
                    let upper_reg = Reg::from_u16(reg_num + 1).unwrap();
                    self.state.reg_write_word(upper_reg, (res >> u16::BITS) as u16);
                }
            },
            Div => {
                let reg_num = ins.reg.to_u16().unwrap();
                assert_eq!(reg_num & 0x1, 0);
                let upper_reg = Reg::from_u16(reg_num + 1).unwrap();
                let upper = self.state.reg_read_word(upper_reg);
                let dividend = ((upper as i32) << u16::BITS) | (reg_val as i32);
                let divisor = src_val as i32;

                if divisor != 0 {
                    let quot = dividend / divisor;
                    let rem = dividend % divisor;
                    if rem != 0 {
                        debug_assert_eq!(rem < 0, dividend < 0);
                    }

                    let res = i16::try_from(quot);
                    self.state.status.set_negative(quot < 0); 
                    self.state.status.set_zero(quot == 0); 
                    self.state.status.set_overflow(src_val == 0 || res.is_err());
                    self.state.status.set_carry(false); 

                    if let Ok(q) = res {
                        // "instruction is aborted" if result can't find in 15 bits
                        // (plus sign, presumably).
                        self.state.reg_write_word(ins.reg, q as u16);
                        assert!(i16::try_from(rem).is_ok());
                        self.state.reg_write_word(upper_reg, rem as u16);
                    }
                } else {
                    self.state.status.set_overflow(true);
                    self.state.status.set_carry(true); 
                }
                
            },
            Ash => {
                const SIG_BITS: u16 = 6;
                const NONSIG_BITS: u16 = (u16::BITS as u16) - SIG_BITS;
                const MASK: u16 = (0x1 << SIG_BITS) - 1;
                let shift = src_val & MASK;
                let mut shift: i16 = ((shift as i16) << NONSIG_BITS) >> NONSIG_BITS; // Sign extend
                assert!((-32i16..=31i16).contains(&shift));

                // Not clear what semantis are for shift more than 16, I'll just clamp.
                shift = shift.clamp(-16, 16);

                let (new_val, carry) = match shift.cmp(&0) {
                    Ordering::Greater => {
                        // Left
                        let carry = if shift < i16::BITS as i16 {
                            ((reg_val >> (i16::BITS  as i16 - shift)) & 0x1) != 0
                        } else {
                            false
                        };
                        let new_val = reg_val << shift;
                        (new_val, carry)
                    },
                    Ordering::Equal => {
                        (reg_val, false)
                    },
                    Ordering::Less => {
                        // Right
                        shift *= -1;
                        let carry = ((reg_val >> (shift - 1)) & 0x1) != 0;
                        let new_val = ((reg_val as i16) >> shift) as u16;
                        (new_val, carry)
                    },
                };
                self.state.reg_write_word(ins.reg, new_val);

                self.state.status.set_negative(Size::Word.sign_bit(new_val as u32) != 0);
                self.state.status.set_zero(new_val == 0);
                self.state.status.set_overflow(
                    (Size::Word.sign_bit(reg_val as u32) != 0) != self.state.status.get_negative()
                );
                self.state.status.set_carry(carry);
                
            },
            Ashc => {
                todo!()
            },
            Xor => unreachable!(),
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

    fn interrupt(&mut self, vector: u16) {
        let old_ps = self.get_state().get_status().to_raw();
        let old_pc = self.state.pc();
        self.push_word(old_ps);
        self.push_word(old_pc);

        let new_pc = self.mem_read_word(vector);
        let new_ps = self.mem_read_word(vector + 2);
        debug!("Interrupt; saving pc {old_pc:#o} and ps {old_ps:#o}; loading pc {new_pc:#o}, ps {new_ps:#o}");
        self.get_state_mut().reg_write_word(Reg::PC, new_pc);
        self.get_state_mut().set_status(Status::from_raw(new_ps));
    }

    fn exec_trap_ins(&mut self, ins: &TrapIns) {
        match ins.op {
            TrapOpcode::Emt => self.interrupt(0o30),
            TrapOpcode::Trap => self.interrupt(0o34),
        }
    }

    fn exec_rti_ins(&mut self) {
        let new_pc = self.pop_word();
        let new_ps = self.pop_word();
        debug!("RTI to pc {new_pc:#o}, ps {new_ps:#o}");
        self.get_state_mut().reg_write_word(Reg::PC, new_pc);
        self.get_state_mut().set_status(Status::from_raw(new_ps));
    }

    fn exec(&mut self, ins: &Ins) -> ExecRet {
        match ins {
            Ins::DoubleOperand(ins) =>  self.exec_double_operand_ins(ins),
            Ins::Branch(ins) => self.exec_branch_ins(ins),
            Ins::Jmp(ins) =>  self.exec_jmp_ins(ins),
            Ins::Jsr(ins) =>  self.exec_jsr_ins(ins),
            Ins::Rts(ins) =>  self.exec_rts_ins(ins),
            Ins::SingleOperand(ins) =>  self.exec_single_operand_ins(ins),
            Ins::Eis(ins) => self.exec_eis_ins(ins),
            Ins::CC(ins) =>  self.exec_cc_ins(ins),
            Ins::Misc(ins) => { return self.exec_misc_ins(ins); },
            Ins::Trap(ins) =>  self.exec_trap_ins(ins),
        }

        ExecRet::Ok
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
        let bin = as_byte_slice(bin);

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
        let bin = as_byte_slice(bin);

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
        let bin = as_byte_slice(bin);

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
        let bin = as_byte_slice(bin);

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
        let bin = as_byte_slice(bin);

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
        let bin = as_byte_slice(bin);

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
        let bin = as_byte_slice(bin);

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

