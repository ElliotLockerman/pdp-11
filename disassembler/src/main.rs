
use disassembler::{Disassembled, disassemble};
use common::constants::WORD_SIZE;

use std::ops::Range;

use clap::Parser;

/// PDP-11 Disassembler
#[derive(Parser)]
struct Args {
    /// Binary to disassemble
    bin: String,
}

fn remove_long_zeros(disassembly: &mut Vec<Disassembled>) {
    const THRESH: usize = 8;

    let mut ranges = vec![];
    let mut range_start = None;
    for (i, dis) in disassembly.iter().enumerate() {
        if dis.repr.len() == 1 && dis.repr[0] == 0 {
            if range_start.is_none() {
                range_start = Some(i);
            }
        } else if let Some(start) = range_start {
            ranges.push(Range{start, end: i});
            range_start = None;
        }       
    }

    for range in ranges.iter().rev() {
        if range.len() > THRESH {
            // Leave the first and last, an ellipses will be added between.
            disassembly.drain(range.start + 1..range.end - 1);
        }
    }
}


fn main() {
    env_logger::init();

    let args = Args::parse();
    let bin = std::fs::read(args.bin).unwrap();
    let mut disassembly = disassemble(&bin);

    remove_long_zeros(&mut disassembly);

    let mut prev: Option<Disassembled> = None;
    for dis in disassembly {
        if let Some(p) = &prev {
            if (p.addr as usize) + p.repr.len() * (WORD_SIZE as usize) != (dis.addr as usize) {
                println!("...");
            }
        }
        println!("{}", dis);
        prev = Some(dis);
    }
}
