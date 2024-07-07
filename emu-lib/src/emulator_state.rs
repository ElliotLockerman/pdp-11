
use crate::constants::*;
use common::asm::{Reg, NUM_REGS};
use common::mem::as_word_slice;

use num_traits::ToPrimitive;

#[derive(Default, Debug)]
pub struct Status(u16);

impl Status {
    const CARRY: usize = 0;
    const OVERFLOW: usize = 1;
    const ZERO: usize = 2;
    const NEGATIVE: usize = 3;
    const T: usize = 4;

    const PRIO: usize = 5;
    const PRIO_MASK: u16 = 0x7;


    pub fn new() -> Status {
        Default::default()
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
        self.0 |= val << Self::PRIO;
    }

    pub fn set_flags(&mut self, bits: u16) {
        assert_eq!(bits & !0xf, 0);
        self.0 |= bits;
    }

    pub fn clear_flags(&mut self, bits: u16) {
        assert_eq!(bits & !0xf, 0);
        self.0 &= !bits;
    }
}



// This is separate so a mutable borrow can be passed to the MMIO handlers.
pub struct EmulatorState {
    curr_cycle: usize,
    mem: Vec<u8>,
    regs: [u16; NUM_REGS],
    pub status: Status,
}

impl EmulatorState {
    pub fn new(mem_size: u16) -> Self {
        assert!(mem_size >= DATA_START);
        assert!(mem_size < DATA_END);
        EmulatorState {
            curr_cycle: 0usize,
            mem: vec![0; mem_size as usize],
            regs: [0; NUM_REGS],
            status: Status::new(),
        }
    }

    pub fn inc_cycle(&mut self) {
        self.curr_cycle += 1;
    }

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
        self.reg_write_word(reg, val as i8 as i16 as u16);
    }

    pub fn pc(&self) -> u16 {
        self.reg_read_word(Reg::PC)
    }

    // Returns next instruction and word after for 
    pub fn next_ins(&self) -> &[u16] {
        let pc = self.pc() as usize;
        let mem = &self.mem.as_slice()[pc..pc+6];
        unsafe { as_word_slice(mem) }
    }
}


