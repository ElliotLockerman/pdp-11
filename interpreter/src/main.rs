
use as_lib::assemble_with_symbols;
use emu_lib::Emulator;
use emu_lib::io::teletype::Teletype;
use emu_lib::io::clock::Clock;

use clap::Parser;

/// PDP-11 Assembly Interpreter
#[derive(Parser)]
#[command(about)]
struct Args {
    /// Input assembly file
    input: String,

    /// Symbol at which to start executing
    #[arg(long, default_value="_start")]
    start: String,

    /// Print symbols.
    #[arg(long)]
    dump_symbols: bool,
}


fn main() {
    env_logger::init();

    let opt = Args::parse();
    let input = std::fs::read_to_string(opt.input).unwrap();
    let (bin, symbols) = assemble_with_symbols(input.as_str());

    if opt.dump_symbols {
        eprintln!("symbols: \n");
        for (k, v) in &symbols {
            eprintln!("{k}:\t{v:#o}")
        }
    }

    let mut emu = Emulator::new();
    emu.set_mmio_handler(Teletype::default());
    emu.set_mmio_handler(Clock::default());

    emu.load_image(&bin, 0);
    let Some(start) = symbols.get(&opt.start) else {
        panic!("Start symbol {} not found", opt.start);
    };
    emu.run_at(*start);
}
