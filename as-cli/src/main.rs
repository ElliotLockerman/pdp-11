
use std::fs::File;
use std::io::Write;

use as_lib::assemble_with_symbols;

use clap::Parser;


/// PDP-11 Assembler
#[derive(Parser)]
#[command(about)]
struct Args {
    /// Input assembly file
    input: String,

    /// File name to output to
    #[arg(long, short)]
    output: Option<String>,

    // Dump symbol table (json).
    #[arg(long)]
    symbols: bool,
}

fn main() {
    env_logger::init();

    let opt = Args::parse();
    let input = std::fs::read_to_string(opt.input).unwrap();
    let (bin, symbols) = assemble_with_symbols(input.as_str());

    let outname = opt.output.as_deref().unwrap_or("a.out");
    let mut out = File::create(outname).unwrap();
    out.write_all(bin.as_slice()).unwrap();

    if opt.symbols {
        println!("{}", serde_json::to_string(&symbols).unwrap());
    }
}

