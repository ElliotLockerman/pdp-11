
#![feature(pointer_is_aligned_to)]

mod emulator;
mod common;

use emulator::Emulator;
use emulator::constants::DATA_END;

use clap::Parser;


/// PDP-11 Emulator
#[derive(Parser)]
struct Args {
    /// Binary to execute
    bin: String,
}



fn main() {
    let opt = Args::parse();
    let mut emu = Emulator::new(DATA_END);

    let buf = std::fs::read(opt.bin).unwrap();
    emu.load_image(buf.as_slice(), 0);

    emu.run();
}


