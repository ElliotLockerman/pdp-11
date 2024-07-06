
use lalrpop_util::lalrpop_mod;

pub mod assembler;
pub mod ir;
// pub mod grammar;
pub mod helpers;

lalrpop_mod!(grammar, "/grammar.rs");

pub use assembler::assemble;

