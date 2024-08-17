
use emu_lib::Emulator;
use emu_lib::emulator_state::Status;

pub const C: u16 = Status::C;
pub const V: u16 = Status::V;
pub const Z: u16 = Status::Z;
pub const N: u16 = Status::N;

pub fn check_flags(emu: &Emulator, exp: u16) {
    assert_eq!(emu.get_carry(), exp & C != 0, "carry flag");
    assert_eq!(emu.get_overflow(),exp & V != 0, "overflow flag");
    assert_eq!(emu.get_zero(), exp & Z != 0, "zero flag");
    assert_eq!(emu.get_negative(), exp & N != 0, "negative flag");
}

