
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use crate::EmulatorState;
use crate::io::{Interrupt, MMIOHandler};

use log::error;

pub trait Printer: Send + Sync {
    fn write(&self, val: u8);
}

#[derive(Default, Clone, Copy)]
struct StdoutPrinter();

impl Printer for StdoutPrinter {
    fn write(&self, val: u8) {
        let mut out = stdout().lock();
        out.write_all(&[val]).unwrap();
        out.flush().unwrap();
    }
}

const STDOUT: StdoutPrinter = StdoutPrinter();


#[derive(Default)]
pub struct PipePrinter {
    buf: Mutex<VecDeque<u8>>,
}

impl Printer for PipePrinter {
    fn write(&self, val: u8) {
        self.buf.lock().unwrap().push_back(val);
    }
}

impl PipePrinter {
    pub fn take(&self) -> VecDeque<u8> {
        std::mem::take(&mut self.buf.lock().unwrap())
    }

    pub fn is_empty(&self) -> bool {
        self.buf.lock().unwrap().is_empty()
    }

    pub fn pop_front(&self) -> Option<u8> {
        self.buf.lock().unwrap().pop_front()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Teleprinter {
    device: Arc<dyn Printer>,
    maintenance_control: bool, // Not used.
    interrupt_enabled: bool,
    interrupted: bool,
    ready: bool,
    ticks_until_ready: usize,
}

impl Default for Teleprinter {
    fn default() -> Self {
        Teleprinter::new_to_stdout()
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
    const PRIO: u8 = 0o4;
    const VECTOR: u16 = 0o64;


    #[allow(unused)]
    const READY_MASK: u8 = 0x1 << Self::READY_SHIFT;

    // Takes 100 ms to type a character.
    // I'm going to arbitrarily choose a fixed 5 us per instruction.
    const DELAY_TICKS: usize = 20_000;

    pub fn new_to_stdout() -> Self {
        Self::new(Arc::new(STDOUT))
    }

    pub fn new(printer: Arc<dyn Printer>) -> Self {
        Teleprinter{
            device: printer,
            maintenance_control: false,
            interrupt_enabled: false,
            interrupted: false,
            ready: true,
            ticks_until_ready: 0,
        }
    }

    fn tps_write(&mut self, val: u8) {
        self.maintenance_control = (val & Self::MAINT_MASK) != 0;
        self.interrupt_enabled = (val & Self::INT_ENB_MASK) != 0;

        // Ignore writes to ready
    }

    fn tpb_write(&mut self, val: u8) {
        if self.ready {
            self.device.write(val);
            self.ticks_until_ready = Self::DELAY_TICKS;
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

    fn tick(&mut self, _: &mut EmulatorState) -> Option<Interrupt> {
        if self.maintenance_control {
            todo!()
        }

        if self.ticks_until_ready == 0 {
            assert!(self.ready);
            if self.interrupt_enabled && !self.interrupted {
                return Some(Interrupt{prio: Self::PRIO, vector: Self::VECTOR});
            }
            return None;
        }

        self.ticks_until_ready -= 1;
        if self.ticks_until_ready == 0 {
            self.ready = true;
            self.interrupted = false;
            if self.interrupt_enabled {
                return Some(Interrupt{prio: Self::PRIO, vector: Self::VECTOR});
            }
        }

        None
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
            Self::TPS_UPPER | Self::TPB_UPPER => (),
            _ => panic!("Teleprinter doesn't handle address {addr:o}"),
        }
    }

    fn write_word(&mut self,  emu: &mut EmulatorState, addr: u16, val: u16) {
       self.write_byte(emu, addr, val as u8);
    }

    fn interrupt_accepted(&mut self) {
        self.interrupted = true;
    }
}
