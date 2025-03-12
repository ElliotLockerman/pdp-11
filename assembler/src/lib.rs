use lalrpop_util::lalrpop_mod;

pub mod assembler;
pub mod helpers;
pub mod ir;

lalrpop_mod!(grammar, "/grammar.rs");

pub use assembler::{assemble, Mode, Program, SymbolType, SymbolValue, Value};
