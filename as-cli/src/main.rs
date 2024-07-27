
use std::fs::File;
use std::io::Write;

use as_lib::assemble;

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
}

fn main() {
    let opt = Args::parse();
    let input = std::fs::read_to_string(opt.input).unwrap();
    let output = assemble(input.as_str());

    let outname = opt.output.as_deref().unwrap_or("a.out");
    let mut out = File::create(outname).unwrap();
    out.write_all(output.as_slice()).unwrap();
}

