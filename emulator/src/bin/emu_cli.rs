use emu_lib::io::clock::Clock;
use emu_lib::io::teletype::Teletype;
use emu_lib::Emulator;
use aout::Aout;

use clap::Parser;

/// PDP-11 Emulator
#[derive(Parser)]
struct Args {
    /// Binary to execute
    bin: String,
}

fn main() {
    env_logger::init();

    let opt = Args::parse();

    let mut emu = Emulator::new();
    emu.set_mmio_handler(Teletype::default());
    emu.set_mmio_handler(Clock::default());

    let mut file = std::fs::File::open(opt.bin).unwrap();
    let aout = Aout::read_from(&mut file);
    emu.load_aout(&aout);
    emu.run_at(aout.entry_point);
}
