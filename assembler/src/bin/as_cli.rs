use std::fs::File;

use as_lib::assemble;

use clap::Parser;
use clap_stdin::FileOrStdin;

/// PDP-11 Assembler
#[derive(Parser)]
#[command(about)]
struct Args {
    /// Input assembly file
    input: FileOrStdin,

    /// File name to output to
    #[arg(long, short)]
    output: Option<String>,
}

fn main() {
    env_logger::init();

    let args = Args::parse();
    let input = args.input.contents().unwrap();
    let prog = assemble(input.as_str());

    let outname = args.output.as_deref().unwrap_or("a.out");
    let mut out = File::create(outname).unwrap();
    prog.write_to(&mut out);
}
