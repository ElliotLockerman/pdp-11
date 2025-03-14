
#![feature(ascii_char)]

pub mod emulator;
pub mod io;
pub mod emulator_state;

pub use emulator::{Emulator, ExecRet};
pub use emulator_state::{EmulatorState, Status};
pub use io::MMIOHandler;

