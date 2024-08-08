

use clap::Parser;

/// PDP-11 Disassembler
#[derive(Parser)]
struct Args {
    /// Binary to disassemble
    bin: String,
}


fn main() {
    env_logger::init();

    let args = Args::parse();
    let bin = std::fs::read(args.bin).unwrap();
    let disassembly = disassembler::disassemble(&bin);
    for dis in disassembly {
        println!("{}", dis);
    }
}
