
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

use crate::EmulatorState;
use crate::io::{Interrupt, MMIOHandler};

use log::error;

pub trait Tty: Send + Sync {
    fn handle_output(&self, val: u8);

    fn input_available(&self) -> bool;
    fn poll_input(&self) -> Option<u8>;
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Clone, Copy)]
struct StdIo();

impl Tty for StdIo {
    fn handle_output(&self, val: u8) {
        let mut out = stdout().lock();
        out.write_all(&[val]).unwrap();
        out.flush().unwrap();
    }

    fn input_available(&self) -> bool {
        todo!()
    }

    fn poll_input(&self) -> Option<u8> {
        todo!()
    }
}

const STDIO: StdIo = StdIo();

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct PipeTty {
    out_buf: Mutex<VecDeque<u8>>,
    in_buf: Mutex<VecDeque<u8>>,
}

impl PipeTty {
    pub fn take_output(&self) -> VecDeque<u8> {
        std::mem::take(&mut self.out_buf.lock().unwrap())
    }

    pub fn is_out_empty(&self) -> bool {
        self.out_buf.lock().unwrap().is_empty()
    }

    pub fn pop_output(&self) -> Option<u8> {
        self.out_buf.lock().unwrap().pop_front()
    }

    pub fn push_input(&self, val: u8) {
        self.in_buf.lock().unwrap().push_back(val);
    }

    pub fn write_input(&self, vals: &[u8]) {
        for val in vals.iter() {
            self.push_input(*val);
        }
    }
}

impl Tty for PipeTty {
    fn handle_output(&self, val: u8) {
        self.out_buf.lock().unwrap().push_back(val);
    }

    fn input_available(&self) -> bool {
        !self.in_buf.lock().unwrap().is_empty()
    }

    fn poll_input(&self) -> Option<u8> {
        self.in_buf.lock().unwrap().pop_front()
    }
}


////////////////////////////////////////////////////////////////////////////////

pub struct Teletype {
    device: Arc<dyn Tty>,

    tps_maintenance_control: bool, // Not used.
    tps_interrupt_enabled: bool,
    printer_interrupted: bool,
    printer_interrupt_accepted: bool,
    tps_ready: bool,
    tps_ticks_until_ready: usize,

    tks_interrupt_enabled: bool,
    keyboard_interrupt_accepted: bool,
    keyboard_interrupted: bool,
}

impl Default for Teletype {
    fn default() -> Self {
        Teletype::new_to_stdout()
    }
}

impl Teletype {
    // TelePrinterStatus
    pub const TPS: u16 = 0o177564;
    const TPS_UPPER: u16 = Self::TPS + 1;

    // TelePrinter Buffer
    pub const TPB: u16 = 0o177566;
    const TPB_UPPER: u16 = Self::TPB + 1;

    const TPS_MAINT_SHIFT: u8 = 2;
    const TPS_MAINT_MASK: u8 = 0x1 << Self::TPS_MAINT_SHIFT;
    const TPS_INT_ENB_SHIFT: u8 = 6;
    const TPS_INT_ENB_MASK: u8 = 0x1 << Self::TPS_INT_ENB_SHIFT;
    const TPS_READY_SHIFT: u8 = 7;
    const PRINT_PRIO: u8 = 0o4;
    const PRINT_VECTOR: u16 = 0o64;

    // Teletype Keyboard Status
    pub const TKS: u16 = 0o177560;
    const TKS_UPPER: u16 = Self::TKS + 1;

    // Teletype Keyboard Buffer
    pub const TKB: u16 = 0o177562;
    const TKB_UPPER: u16 = Self::TKS + 1;

    // Only used for paper tape reader (not yet supported)
    const TKS_RDR_ENB_SHIFT: u16 = 0;
    #[allow(dead_code)]
    const TKS_RDR_ENB_MASK: u16 = 1u16 << Self::TKS_RDR_ENB_SHIFT;
    const TKS_BUSY_SHIFT: u16 = 11;
    #[allow(dead_code)]
    const TKS_BUSY_MASK: u16 = 0x1 << Self::TKS_BUSY_SHIFT;

    const TKS_INT_ENB_SHIFT: u16 = 6;
    #[allow(dead_code)]
    const TKS_INT_ENB_MASK: u16 = 0x1 << Self::TKS_INT_ENB_SHIFT;

    const TKS_DONE_SHIFT: u16 = 7;
    #[allow(dead_code)]
    const TKS_DONE_MASK: u16 = 0x1 << Self::TKS_DONE_SHIFT;

    #[allow(dead_code)]
    const KEY_PRIO: u8 = 0o4;
    #[allow(dead_code)]
    const KEY_VECTOR: u16 = 0o60;



    #[allow(unused)]
    const TPS_READY_MASK: u8 = 0x1 << Self::TPS_READY_SHIFT;

    // Takes 100 ms to type a character.
    // I'm going to arbitrarily choose a fixed 5 us per instruction.
    const PRINT_DELAY_TICKS: usize = 20_000;

    pub fn new_to_stdout() -> Self {
        Self::new(Arc::new(STDIO))
    }

    pub fn new(device: Arc<dyn Tty>) -> Self {
        Teletype{
            device,

            tps_maintenance_control: false,
            tps_interrupt_enabled: false,
            printer_interrupted: false,
            printer_interrupt_accepted: false,
            tps_ready: true,
            tps_ticks_until_ready: 0,

            tks_interrupt_enabled: false,
            keyboard_interrupted: false,
            keyboard_interrupt_accepted: false,
        }
    }

    fn tps_write(&mut self, val: u8) {
        self.tps_maintenance_control = (val & Self::TPS_MAINT_MASK) != 0;
        self.tps_interrupt_enabled = (val & Self::TPS_INT_ENB_MASK) != 0;

        // Ignore writes to ready
    }

    fn tpb_write(&mut self, val: u8) {
        if self.tps_ready {
            self.device.handle_output(val);
            self.tps_ticks_until_ready = Self::PRINT_DELAY_TICKS;
            self.tps_ready = false;
        } else {
            error!("Teletype: write to TPB of {val} when not ready");
        }
    }

    fn tps_read(&self) -> u8 {
        ((self.tps_maintenance_control as u8) << Self::TPS_MAINT_SHIFT)
            | ((self.tps_interrupt_enabled as u8) << Self::TPS_INT_ENB_SHIFT)
            | ((self.tps_ready as u8) << Self::TPS_READY_SHIFT)
    }

    fn tks_write(&mut self, val: u16) {
        self.tks_interrupt_enabled = (val & Self::TKS_INT_ENB_MASK) != 0;
    }

    fn tks_read(&mut self) -> u16 {
        // BUSY, RDR ENB not used yet, always 0.
        ((self.tps_interrupt_enabled as u16) << Self::TPS_INT_ENB_SHIFT)
            | ((self.device.input_available() as u16) << Self::TKS_DONE_SHIFT)
    }

    fn tkb_read(&mut self) -> u8 {
        if let Some(ch) = self.device.poll_input() {
            self.keyboard_interrupt_accepted = false;
            println!("keyboard returned '{}'", ch as char);
            return ch;
        }
        error!("Teletype: read of TKB when no character is available");
        0
    }
}

impl MMIOHandler for Teletype {

    fn tick(&mut self, _: &mut EmulatorState) -> Option<Interrupt> {
        if self.tps_maintenance_control {
            todo!()
        }

        // Keyboard gets priority.
        if self.device.input_available()
            && self.tks_interrupt_enabled
            && !self.printer_interrupt_accepted {

            self.keyboard_interrupted = true;
            return Some(Interrupt{prio: Self::PRINT_PRIO, vector: Self::KEY_VECTOR});
        }

        if self.tps_ticks_until_ready == 0 {
            assert!(self.tps_ready);
            if self.tps_interrupt_enabled && !self.printer_interrupt_accepted {
                self.printer_interrupted = true;
                return Some(Interrupt{prio: Self::PRINT_PRIO, vector: Self::PRINT_VECTOR});
            }
            return None;
        }

        self.tps_ticks_until_ready -= 1;
        if self.tps_ticks_until_ready == 0 {
            self.tps_ready = true;
            self.printer_interrupt_accepted = false;
            if self.tps_interrupt_enabled {
                self.printer_interrupted = true;
                return Some(Interrupt{prio: Self::PRINT_PRIO, vector: Self::PRINT_VECTOR});
            }
        }

        None
    }

    fn read_byte(&mut self, _: &mut EmulatorState, addr: u16) -> u8 {
        match addr {
            Self::TPS => self.tps_read(),
            Self::TPS_UPPER | Self::TPB | Self::TPB_UPPER => 0,
            Self::TKS => self.tks_read() as u8,
            Self::TKS_UPPER => (self.tks_read() >> u8::BITS) as u8,
            Self::TKB => self.tkb_read(),
            _ => panic!("Teletype doesn't handle address {addr:o}"),
        }
    }

    fn read_word(&mut self, emu: &mut EmulatorState, addr: u16) -> u16 {
        if addr == Self::TKS {
            self.tks_read()
        } else {
            self.read_byte(emu, addr) as u16
        }
    }

    fn write_byte(&mut self, _: &mut EmulatorState, addr: u16, val: u8) {
        match addr {
            Self::TPS => self.tps_write(val),
            Self::TPB => self.tpb_write(val),
            Self::TKS => self.tks_write(val as u16),
            Self::TPS_UPPER | Self::TPB_UPPER | Self::TKB | Self::TKB_UPPER => (),
            _ => panic!("Teletype doesn't handle address {addr:o}"),
        }
    }

    fn write_word(&mut self,  emu: &mut EmulatorState, addr: u16, val: u16) {
       self.write_byte(emu, addr, val as u8);
    }

    fn interrupt_accepted(&mut self) {
        if self.keyboard_interrupted {
            self.keyboard_interrupted = false;
            self.keyboard_interrupt_accepted = true;
        } else if self.printer_interrupted {
            self.printer_interrupted = false;
            self.printer_interrupt_accepted = true;
        } else {
            panic!("Teletype received interrupt_accepted() but didn't interrupt");
        }
    }

    fn default_addrs(&self) -> &[u16] {
        &[Self::TPS, Self::TPB, Self::TKS, Self::TKB]
    }
}
