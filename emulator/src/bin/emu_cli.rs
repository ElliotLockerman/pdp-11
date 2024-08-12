
use emu_lib::Emulator;
use emu_lib::io::teleprinter::Teleprinter;
use emu_lib::io::clock::Clock;

use clap::Parser;


/// PDP-11 Emulator
#[derive(Parser)]
struct Args {
    /// Binary to execute
    bin: String,

    /// Address at which to start executing.
    #[arg(long, default_value_t=0)]
    start: u16,
}



fn main() {
    env_logger::init();

    let opt = Args::parse();

    let mut emu = Emulator::new();
    emu.set_mmio_handler(Teleprinter::default());
    emu.set_mmio_handler(Clock::default());

    let buf = std::fs::read(opt.bin).unwrap();
    emu.load_image(buf.as_slice(), 0);

    emu.run_at(opt.start);
}


