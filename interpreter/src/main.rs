use as_lib::assemble;
use emu_lib::Emulator;
use emu_lib::io::clock::Clock;
use emu_lib::io::teletype::Teletype;

use clap::Parser;

/// PDP-11 Assembly Interpreter
#[derive(Parser)]
#[command(about)]
struct Args {
    /// Input assembly file
    input: String,
}

fn main() {
    env_logger::init();

    let opt = Args::parse();
    let input = std::fs::read_to_string(opt.input).unwrap();
    let aout = assemble(input.as_str());

    let mut emu = Emulator::new();
    emu.set_mmio_handler(Teletype::default());
    emu.set_mmio_handler(Clock::default());

    emu.load_aout(&aout);
    emu.run_at(aout.entry_point);
}
