#![cfg(test)]
#![feature(bigint_helper_methods)]
#![feature(assert_matches)]

mod flags;

mod addressing_modes;
mod mixed_addressing;
mod double_operand;
mod single_operand;
mod eis;
mod condition_code;
mod branch;
mod jmp;
mod call;
mod progs;
mod io;
mod exprs;
mod misc;
mod trap;

