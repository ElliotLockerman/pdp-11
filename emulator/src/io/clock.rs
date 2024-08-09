use crate::EmulatorState;
use crate::io::{Interrupt, MMIOHandler};

pub struct Clock {
    interrupt_enable: bool,
    clock: bool,
    ticks_until_ready: usize,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            interrupt_enable: false,
            clock: false,
            ticks_until_ready: Self::DELAY_TICKS,
        }
    }
}

impl Clock {

    pub const LKS: u16 = 0o177546;
    pub const LKS_UPPER: u16 = 0o177547;
    pub const ADDRS: &[u16] = &[Self::LKS];

    const INT_ENB_SHIFT: u8 = 6;
    const CLOCK_SHIFT: u8 = 7;
    const PRIO: u8 = 0o6;
    const VECTOR: u16 = 0o100;

    // Ticks every 16.6 ms
    // I'm going to arbitrarily choose a fixed 5 us per instruction.
    const DELAY_TICKS: usize = 3_320;

    #[allow(dead_code)]
    fn new() -> Self {
        Self::default()
    }

    fn lks_write(&mut self, val: u8) {
        if (val >> Self::INT_ENB_SHIFT) & 0x1 != 0 {
            self.interrupt_enable = true;
        }
    }

    fn lks_read(&mut self) -> u8 {
        let val = ((self.interrupt_enable as u8) << Self::INT_ENB_SHIFT)
            | ((self.clock as u8) << Self::CLOCK_SHIFT);
        self.clock = false;
        val
    }
}

impl MMIOHandler for Clock {
    fn reset(&mut self, _emu: &mut EmulatorState) {
        self.interrupt_enable = false;
        self.clock = false;
        self.ticks_until_ready = Self::DELAY_TICKS;
    }

    fn tick(&mut self, _emu: &mut EmulatorState) -> Option<Interrupt> {
        self.ticks_until_ready -= 1;
        if self.ticks_until_ready == 0 {
            self.clock = true;
            self.ticks_until_ready = Self::DELAY_TICKS;
        }

        if self.clock && self.interrupt_enable {
            Some(Interrupt{prio: Self::PRIO, vector: Self::VECTOR})
        } else {
            None
        }
    }

    fn read_byte(&mut self, _: &mut EmulatorState, addr: u16) -> u8 {
        match addr {
            Self::LKS => self.lks_read(),
            Self::LKS_UPPER => 0,
            _ => panic!("Clock doesn't handle address {addr:o}"),
        }
    }

    fn read_word(&mut self, emu: &mut EmulatorState, addr: u16) -> u16 {
        self.read_byte(emu, addr) as u16
    }

    fn write_byte(&mut self, _: &mut EmulatorState, addr: u16, val: u8) {
        match addr {
            Self::LKS => self.lks_write(val),
            Self::LKS_UPPER => (),
            _ => panic!("Clock doesn't handle address {addr:o}"),
        }
    }

    fn write_word(&mut self,  emu: &mut EmulatorState, addr: u16, val: u16) {
       self.write_byte(emu, addr, val as u8);
    }
}
