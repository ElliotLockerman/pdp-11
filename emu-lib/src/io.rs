
use crate::EmulatorState;


pub trait MMIOHandler {
    fn reset(&mut self);

    fn read_byte(&mut self, emu: &mut EmulatorState, addr: u16) -> u8;
    fn read_word(&mut self, emu: &mut EmulatorState, addr: u16) -> u16;

    fn write_byte(&mut self,  emu: &mut EmulatorState, addr: u16, val: u8);
    fn write_word(&mut self,  emu: &mut EmulatorState, addr: u16, val: u16);
}


