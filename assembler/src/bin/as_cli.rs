use std::fs::File;

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
    env_logger::init();

    let opt = Args::parse();
    let input = std::fs::read_to_string(opt.input).unwrap();
    let prog = assemble(input.as_str());

    let outname = opt.output.as_deref().unwrap_or("a.out");
    let mut out = File::create(outname).unwrap();
    prog.write_to(&mut out);
}
