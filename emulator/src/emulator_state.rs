use common::asm::{NUM_REGS, Reg};

use bytemuck::cast_slice;
use log::trace;
use num_traits::ToPrimitive;

#[derive(Default, Debug)]
pub struct Status(u16);

impl Status {
    pub const CARRY_SHIFT: u16 = 0;
    pub const OVERFLOW_SHIFT: u16 = 1;
    pub const ZERO_SHIFT: u16 = 2;
    pub const NEGATIVE_SHIFT: u16 = 3;
    const T: u16 = 4;

    pub const C: u16 = 0x1 << Self::CARRY_SHIFT;
    pub const V: u16 = 0x1 << Self::OVERFLOW_SHIFT;
    pub const Z: u16 = 0x1 << Self::ZERO_SHIFT;
    pub const N: u16 = 0x1 << Self::NEGATIVE_SHIFT;

    const FLAGS_MASK: u16 = 0xf;

    const PRIO: u16 = 5;
    const PRIO_MASK: u16 = 0x7;

    pub fn new() -> Status {
        Default::default()
    }

    pub fn from_raw(raw: u16) -> Self {
        Status(raw)
    }

    pub fn to_raw(&self) -> u16 {
        self.0
    }

    pub fn get_flags(&self) -> u16 {
        self.0 & Self::FLAGS_MASK
    }

    pub fn set_flags(&mut self, bits: u16) {
        assert_eq!(bits & !Self::FLAGS_MASK, 0);
        self.0 &= !Self::FLAGS_MASK;
        self.0 |= bits;
    }

    pub fn get_carry(&self) -> bool {
        (self.0 & Self::C) != 0
    }

    pub fn set_carry(&mut self, val: bool) {
        self.0 &= !(1u16 << Self::CARRY_SHIFT);
        self.0 |= (val as u16) << Self::CARRY_SHIFT;
    }

    pub fn get_overflow(&self) -> bool {
        (self.0 & Self::V) != 0
    }

    pub fn set_overflow(&mut self, val: bool) {
        self.0 &= !(1u16 << Self::OVERFLOW_SHIFT);
        self.0 |= (val as u16) << Self::OVERFLOW_SHIFT;
    }

    pub fn get_zero(&self) -> bool {
        (self.0 & Self::Z) != 0
    }

    pub fn set_zero(&mut self, val: bool) {
        self.0 &= !(1u16 << Self::ZERO_SHIFT);
        self.0 |= (val as u16) << Self::ZERO_SHIFT;
    }

    pub fn get_negative(&self) -> bool {
        (self.0 & Self::N) != 0
    }

    pub fn set_negative(&mut self, val: bool) {
        self.0 &= !(1u16 << Self::NEGATIVE_SHIFT);
        self.0 |= (val as u16) << Self::NEGATIVE_SHIFT;
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
        self.0 |= val << Self::PRIO;
    }
}

// This is separate so a mutable borrow can be passed to the MMIO handlers.
pub struct EmulatorState {
    num_ins: usize,
    mem: Vec<u8>,
    regs: [u16; NUM_REGS],
    status: Status,
}

impl EmulatorState {
    pub fn new() -> Self {
        EmulatorState {
            num_ins: 0usize,
            mem: vec![0; (u16::MAX as usize) + 1],
            regs: [0; NUM_REGS],
            status: Status::new(),
        }
    }

    pub fn inc_ins(&mut self) {
        self.num_ins += 1;
    }

    pub fn mem_read_byte(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn mem_write_byte(&mut self, addr: u16, val: u8) {
        trace!("Mem: writing {val:#o} to 0o{addr:o} (byte)");
        self.mem[addr as usize] = val;
    }

    pub fn mem_read_word(&self, addr: u16) -> u16 {
        assert!(addr & 1 == 0);
        (self.mem[addr as usize] as u16) | ((self.mem[(addr + 1) as usize] as u16) << 8)
    }

    pub fn mem_write_word(&mut self, addr: u16, val: u16) {
        trace!("Mem: writing {val:#o} to 0o{addr:o} (word)");
        assert!(addr & 1 == 0);
        self.mem[addr as usize] = val as u8;
        self.mem[(addr + 1) as usize] = (val >> 8) as u8;
    }

    pub fn reg_write_word(&mut self, reg: Reg, val: u16) {
        trace!("Reg: writing {val:#o} to {reg:?} (word)");
        if reg == Reg::SP && val < 0o400 {
            // Should really trap.
            panic!("Stack overflow");
        }
        self.regs[reg.to_usize().unwrap()] = val;
    }

    pub fn reg_read_word(&self, reg: Reg) -> u16 {
        self.regs[reg.to_usize().unwrap()]
    }

    pub fn reg_read_byte(&self, reg: Reg) -> u8 {
        self.reg_read_word(reg) as u8
    }

    pub fn reg_write_byte(&mut self, reg: Reg, val: u8) {
        trace!("Reg: writing {val:#o} to {reg:?} (byte)");
        let mut old = self.reg_read_word(reg);
        old &= !0xff;
        old |= val as u16;
        self.reg_write_word(reg, old);
    }

    pub fn pc(&self) -> u16 {
        self.reg_read_word(Reg::PC)
    }

    // Returns next instruction and word after for
    pub fn next_ins(&self) -> &[u16] {
        let pc = self.pc() as usize;
        if pc & 0x1 != 0 {
            panic!("PC 0o{pc:o} not aligned");
        }
        let mem = &self.mem.as_slice()[pc..pc + 6];
        cast_slice(mem)
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn get_status(&self) -> &Status {
        &self.status
    }

    pub fn get_status_mut(&mut self) -> &mut Status {
        &mut self.status
    }
}

impl Default for EmulatorState {
    fn default() -> Self {
        Self::new()
    }
}
