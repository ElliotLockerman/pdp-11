
use std::io::{stdout, Write};

use crate::EmulatorState;

use log::error;


pub trait MMIOHandler {
    fn cycle(&mut self, _emu: &mut EmulatorState) {}

    fn read_byte(&mut self, emu: &mut EmulatorState, addr: u16) -> u8;
    fn read_word(&mut self, emu: &mut EmulatorState, addr: u16) -> u16;

    fn write_byte(&mut self,  emu: &mut EmulatorState, addr: u16, val: u8);
    fn write_word(&mut self,  emu: &mut EmulatorState, addr: u16, val: u16);
}

////////////////////////////////////////////////////////////////////////////////

pub struct Teleprinter {
    maintenance_control: bool, // Not used.
    interrupt_enabled: bool, // Not yet used.
    ready: bool,
    cycles_until_ready: usize,
}

impl Default for Teleprinter {
    fn default() -> Self {
        Teleprinter{
            maintenance_control: false,
            interrupt_enabled: false,
            ready: true,
            cycles_until_ready: 0,
        }
    }
}

impl Teleprinter {
    // TelePrinterStatus
    pub const TPS: u16 = 0o177564;
    const TPS_UPPER: u16 = 0o177565;

    // Teleprinter Buffer
    pub const TPB: u16 = 0o177566;
    const TPB_UPPER: u16 = 0o177567;

    const MAINT_SHIFT: u8 = 2;
    const MAINT_MASK: u8 = 0x1 << Self::MAINT_SHIFT;
    const INT_ENB_SHIFT: u8 = 6;
    const INT_ENB_MASK: u8 = 0x1 << Self::INT_ENB_SHIFT;
    const READY_SHIFT: u8 = 7;

    #[allow(unused)]
    const READY_MASK: u8 = 0x1 << Self::READY_SHIFT;

    const DELAY_CYCLES: usize = 125_000;

    pub fn new() -> Self {
        Default::default()
    }

    fn tps_write(&mut self, val: u8) {
        if val & Self::MAINT_MASK != 0 {
            self.maintenance_control = true;
        }

        if val & Self::INT_ENB_MASK != 0 {
            todo!()
        }

        // Ignore writes to ready
    }

    fn tpb_write(&mut self, val: u8) {
        if self.ready {
            let mut out = stdout().lock();
            out.write_all(&[val]).unwrap();
            out.flush().unwrap();
            self.cycles_until_ready = Self::DELAY_CYCLES;
            self.ready = false;
        } else {
            error!("Teleprinter: write to TPB of {val} when not ready");
        }
    }

    fn tps_read(&self) -> u8 {
        ((self.maintenance_control as u8) << Self::MAINT_SHIFT)
            | ((self.interrupt_enabled as u8) << Self::INT_ENB_SHIFT)
            | ((self.ready as u8) << Self::READY_SHIFT)
    }
}

impl MMIOHandler for Teleprinter {

    fn cycle(&mut self, _: &mut EmulatorState) {
        if self.maintenance_control {
            todo!()
        }

        if self.cycles_until_ready == 0 {
            return;
        }

        self.cycles_until_ready -= 1;
        if self.cycles_until_ready == 0 {
            self.ready = true;
        }
    }

    fn read_byte(&mut self, _: &mut EmulatorState, addr: u16) -> u8 {
        match addr {
            Self::TPS => self.tps_read(),
            Self::TPS_UPPER | Self::TPB | Self::TPB_UPPER => 0,
            _ => panic!("Teleprinter doesn't handle address {addr:o}"),
        }
    }

    fn read_word(&mut self, emu: &mut EmulatorState, addr: u16) -> u16 {
        self.read_byte(emu, addr) as u16
    }

    fn write_byte(&mut self, _: &mut EmulatorState, addr: u16, val: u8) {
        match addr {
            Self::TPS => self.tps_write(val),
            Self::TPB => self.tpb_write(val),
            Self::TPS_UPPER | Self::TPB_UPPER => return,
            _ => panic!("Teleprinter doesn't handle address {addr:o}"),
        }
    }

    fn write_word(&mut self,  emu: &mut EmulatorState, addr: u16, val: u16) {
       self.write_byte(emu, addr, val as u8);
    }
}

