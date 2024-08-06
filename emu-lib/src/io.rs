
pub mod teleprinter;
pub mod clock;

use crate::EmulatorState;

#[derive(Debug, Clone, Copy)]
pub struct Interrupt {
    pub prio: u8, // 0o0 through 0o7
    pub vector: u16,
}

pub trait MMIOHandler {
    fn reset(&mut self, _emu: &mut EmulatorState) {}
    fn tick(&mut self, _emu: &mut EmulatorState) -> Option<Interrupt> { None }

    fn read_byte(&mut self, emu: &mut EmulatorState, addr: u16) -> u8;
    fn read_word(&mut self, emu: &mut EmulatorState, addr: u16) -> u16;

    fn write_byte(&mut self,  emu: &mut EmulatorState, addr: u16, val: u8);
    fn write_word(&mut self,  emu: &mut EmulatorState, addr: u16, val: u16);
}


