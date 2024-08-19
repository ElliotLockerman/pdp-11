
use crate::{EmulatorState, Status};
use crate::io::MMIOHandler;

// Accesss for processor status word through MMIO
#[derive(Default, Clone, Copy)]
pub struct StatusAccess();

impl StatusAccess {
    const ADDR: u16 = 0o177776;
    const ADDR_UPPER: u16 = Self::ADDR + 1;
}

impl MMIOHandler for StatusAccess {
    fn read_word(&mut self, state: &mut EmulatorState, addr: u16) -> u16 {
        assert_eq!(addr, Self::ADDR);
        state.get_status().to_raw()
    }

    fn read_byte(&mut self, state: &mut EmulatorState, addr: u16) -> u8 {
        match addr {
            Self::ADDR => state.get_status().to_raw() as u8,
            Self::ADDR_UPPER => 0u8,
            _ => panic!("PsAcesss doesn't handle address {addr:o}"),
        }
    }

    fn write_word(&mut self,  state: &mut EmulatorState, addr: u16, val: u16) {
        assert_eq!(addr, Self::ADDR);
        assert_eq!(val & !0xff, 0);
        state.set_status(Status::from_raw(val));
    }

    fn write_byte(&mut self, state: &mut EmulatorState, addr: u16, val: u8) {
        match addr {
            Self::ADDR => self.write_word(state, addr, val as u16),
            Self::ADDR_UPPER => (),
            _ => panic!("PsAcesss doesn't handle address {addr:o}"),
        }
    }

    fn default_addrs(&self) -> &[u16] {
        &[Self::ADDR]
    }
}


