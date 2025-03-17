use common::asm::*;

use thiserror::Error;

// The Unix v6 manual calls this a "type".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Undef,
    UndefExt,
    Abs,  // Relocatable
    Text, // Relocatable
    Data, // Relocatable
    Bss,  // Relocatable
    Ext,  // External absolute, text, data or bss
    Reg,
}

impl Mode {
    // Returns None if illegal.
    pub fn op_mode(lhs: Mode, op: Op, rhs: Mode) -> Option<Mode> {
        use Mode::*;
        Some(match (lhs, op, rhs) {
            (Undef, _, _) => Undef,
            (_, _, Undef) => Undef,
            (Abs, _, Abs) => Abs,

            (UndefExt, Op::Add, Abs) => UndefExt,
            (Text, Op::Add, Abs) => Text,
            (Data, Op::Add, Abs) => Data,
            (Bss, Op::Add, Abs) => Bss,

            (Text, Op::Sub, Abs) => Text,
            (Data, Op::Sub, Abs) => Data,
            (Bss, Op::Sub, Abs) => Bss,

            (Text, Op::Sub, Text) => Abs,
            (Data, Op::Sub, Data) => Abs,
            (Bss, Op::Sub, Bss) => Abs,

            _ => return None,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub struct Value {
    pub val: u16,
    pub mode: Mode,
}

impl Value {
    pub fn new(val: u16, mode: Mode) -> Self {
        Value { val, mode }
    }
}

impl std::ops::Add<Value> for Value {
    type Output = Result<Value, EvalError>;
    fn add(self, rhs: Value) -> Self::Output {
        let mode = Mode::op_mode(self.mode, Op::Add, rhs.mode).ok_or(EvalError::IllegalExpr(
            self,
            Op::Add,
            rhs,
        ))?;
        Ok(Value {
            val: self.val.wrapping_add(rhs.val),
            mode,
        })
    }
}

impl std::ops::Sub<Value> for Value {
    type Output = Result<Value, EvalError>;
    fn sub(self, rhs: Value) -> Self::Output {
        let mode = Mode::op_mode(self.mode, Op::Sub, rhs.mode).ok_or(EvalError::IllegalExpr(
            self,
            Op::Sub,
            rhs,
        ))?;
        Ok(Value {
            val: self.val.wrapping_sub(rhs.val),
            mode,
        })
    }
}

impl std::ops::BitAnd<Value> for Value {
    type Output = Result<Value, EvalError>;
    fn bitand(self, rhs: Value) -> Self::Output {
        let mode = Mode::op_mode(self.mode, Op::And, rhs.mode).ok_or(EvalError::IllegalExpr(
            self,
            Op::And,
            rhs,
        ))?;
        Ok(Value {
            val: self.val & rhs.val,
            mode,
        })
    }
}

impl std::ops::BitOr<Value> for Value {
    type Output = Result<Value, EvalError>;
    fn bitor(self, rhs: Value) -> Self::Output {
        let mode = Mode::op_mode(self.mode, Op::Or, rhs.mode).ok_or(EvalError::IllegalExpr(
            self,
            Op::Or,
            rhs,
        ))?;
        Ok(Value {
            val: self.val | rhs.val,
            mode,
        })
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("Unable to resolve symbol")]
    SymbolUnresolved,

    #[error("Illegal Expr: {0:?} {} {2:?}", .1)]
    IllegalExpr(Value, Op, Value),
}
