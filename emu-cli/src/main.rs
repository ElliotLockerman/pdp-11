
use emu_lib::{Emulator, constants::DATA_END};
use emu_lib::io::Teleprinter;

use clap::Parser;


/// PDP-11 Emulator
#[derive(Parser)]
struct Args {
    /// Binary to execute
    bin: String,
}



fn main() {
    let opt = Args::parse();

    let teleprinter = Teleprinter::new();
    let mut emu = Emulator::new(DATA_END);
    emu.set_mmio_handler([Teleprinter::TPS, Teleprinter::TPB], teleprinter);

    let buf = std::fs::read(opt.bin).unwrap();
    emu.load_image(buf.as_slice(), 0);

    emu.run();
}


