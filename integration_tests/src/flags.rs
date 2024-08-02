
use emu_lib::emulator_state::Status;

// Because each test is run on a fresh emulator, unaffected flags will be false
#[derive(Debug, Clone, Copy, Default)]
pub struct Flags {
    pub c: bool,
    pub v: bool, // overflow
    pub z: bool,
    pub n: bool,
}

impl Flags {
    pub fn to_bits(self) -> u16 {
        ((self.c as u16) << Status::CARRY)
            | ((self.v as u16) << Status::OVERFLOW) 
            | ((self.z as u16) << Status::ZERO) 
            | ((self.n as u16) << Status::NEGATIVE)
    }

    pub fn c(mut self) -> Self {
        self.c = true;
        self
    }

    pub fn v(mut self) -> Self {
        self.v = true;
        self
    }

    pub fn z(mut self) -> Self {
        self.z = true;
        self
    }

    pub fn n(mut self) -> Self {
        self.n = true;
        self
    }
}

pub fn flags() -> Flags {
    Flags::default()
}

