
#![feature(pointer_is_aligned_to)]

mod assembler;
mod common;

use std::fs::File;
use std::io::Write;

use crate::assembler::assemble;

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

#[cfg(test)]
mod tests {
    use crate::assembler::assemble;
    fn to_u16(arr: &Vec<u8>) -> Vec<u16> {
        assert_eq!(arr.len() % 2, 0);
        let mut out = Vec::new();
        for chunk in arr.chunks(2) {
            out.push((chunk[0] as u16) | (chunk[1] as u16) << 8);
        }
        out
    }

    #[test]
    fn halt() {
        let prog = "halt";
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0);
    }

    #[test]
    fn mov_reg_reg() {
        let prog = "mov r0, r1";
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o10001);
    }

    #[test]
    fn mov_mem_mem() {
        let prog = "mov (r0)+, -(r1)";
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o12041);
    }

    #[test]
    fn branch() {
        let prog = r#"
            label:
                br label"#;
        let bin = to_u16(&assemble(prog));
        assert_eq!(bin.len(), 1);
        assert_eq!(bin[0], 0o000777);
    }
}
