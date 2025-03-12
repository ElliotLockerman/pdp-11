#![cfg(test)]
#![feature(bigint_helper_methods)]
#![feature(assert_matches)]

mod flags;

mod addressing_modes;
mod branch;
mod call;
mod condition_code;
mod double_operand;
mod eis;
mod exprs;
mod io;
mod jmp;
mod misc;
mod mixed_addressing;
mod progs;
mod single_operand;
mod trap;
