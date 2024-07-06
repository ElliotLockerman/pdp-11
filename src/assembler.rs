
use lalrpop_util::lalrpop_mod;

pub mod assemble;
pub mod ir;
// pub mod grammar;
pub mod helpers;

lalrpop_mod!(grammar, "/assembler/grammar.rs");

pub use assemble::assemble;

