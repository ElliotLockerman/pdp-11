
use crate::constants::WORD_SIZE;

use num_derive::{FromPrimitive, ToPrimitive};    
use num_traits::{FromPrimitive, ToPrimitive};    


pub trait InstrVariant<Opcode: FromPrimitive> {
    const OPCODE_BITS: usize;
    const LOWER_BITS: usize;

    fn decode_opcode(input: u16) -> Option<Opcode> {
        let op = input >> Self::LOWER_BITS;
        Opcode::from_u16(op)
    }
}



////////////////////////////////////////////////////////////////////////////////


#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum AddrMode {
    Gen = 0,
    Def, // Deferred (indirect)
    AutoInc,
    AutoIncDef,
    AutoDec,
    AutoDecDef,
    Index,
    IndexDef,

}

impl AddrMode {
    pub const NUM_BITS: usize = 3;
    pub const MASK: u16 = (1u16 << Self::NUM_BITS) - 1;
}

#[derive(Debug, Clone)]
pub enum Extra {
    None,
    Imm(u16),
    Rel(u16),
    LabelRef(String),
}

impl Extra {
    pub fn is_some(&self) -> bool {
        !std::matches!(self, Extra::None)
    }

    pub fn unwrap_imm(&self) -> u16 {
        if let Extra::Imm(val) = self {
            return *val;
        }
        panic!("Extra::unwrap_imm() called on non-imm: {self:?}");
    }

    pub fn is_imm(&self) -> bool {
        matches!{self, Extra::Imm(_)}
    }

    pub fn unwrap_label_ref(&self) -> &String {
        if let Extra::LabelRef(val) = self {
            return val;
        }
        panic!("Extra::unwrap_label_ref() called on non-label-ref: {self:?}");
    }

    pub fn is_label_ref(&self) -> bool {
        matches!{self, Extra::LabelRef(_)}
    }

    pub fn unwrap_rel(&self) -> u16 {
        if let Extra::Rel(val) = self {
            return *val;
        }
        panic!("Extra::unwrap_rel() called on non-rel: {self:?}");
    }

    pub fn is_rel(&self) -> bool {
        matches!{self, Extra::Rel(_)}
    }

    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Extra::None)
    }
}


#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum Reg {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    SP,
    PC,
}

pub const NUM_REGS: usize = 8;

impl Reg {
    pub const NUM_BITS: usize = 3;
    pub const MASK: u16 = (1u16 << Self::NUM_BITS) - 1;
}

#[derive(Debug, Clone)]
pub struct RegArg {
    pub mode: AddrMode,
    pub reg: Reg,
    pub extra: Extra, // Only used by assembler
}

impl RegArg {
    pub const NUM_BITS: usize = AddrMode::NUM_BITS + Reg::NUM_BITS;

    #[allow(dead_code)]
    pub const MASK: u16 = (1u16 << Self::NUM_BITS) - 1;

    pub fn new(mode: AddrMode, reg: Reg, extra: Extra) -> RegArg {
        let ret = RegArg{mode, reg, extra};
        assert!(!ret.has_imm() || ret.extra.is_some());
        ret
    }

    pub fn has_imm(&self) -> bool {
        use AddrMode::*;
        match self.mode {
            AutoInc | AutoIncDef => self.reg == Reg::PC,
            Index | IndexDef => true,
            _ => false,
        }
    }

    pub fn num_imm(&self) -> u16 {
        self.has_imm() as u16
    }


    pub fn format(&self) -> u16 {
        self.reg.to_u16().unwrap() | (self.mode.to_u16().unwrap() << Reg::NUM_BITS)
    }
}


#[derive(Debug, Clone)]
pub enum Target {
    Label(String), // Only used by assembler
    Offset(u8),
}

impl Target {
    pub fn unwrap_offset(&self) -> u8 {
        match self {
            Target::Label(_) => panic!("Target::unwrap_reolved() no resolved"),
            Target::Offset(val) => *val,
        }
    }
}



////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! make_double_operand_ins {
    ($op:ident, $src:expr, $dst:expr) => { Ins::DoubleOperand(DoubleOperandIns{op: DoubleOperandOpcode::$op, src: $src, dst: $dst}) };
}

// Also double operand byte inpub structions
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum DoubleOperandOpcode {
    Mov = 1,
    Cmp,
    Bit,
    Bic,
    Bis,
    Add,

    MovB = 9,
    CmpB,
    BitB,
    BicB,
    BisB,
    Sub,
}



#[derive(Debug, Clone)]
pub struct DoubleOperandIns {
    pub op: DoubleOperandOpcode,
    pub src: RegArg,
    pub dst: RegArg,
}


impl InstrVariant<DoubleOperandOpcode> for DoubleOperandIns {
    const OPCODE_BITS: usize = 4;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

impl DoubleOperandIns {
    pub fn num_imm(&self) -> u16 {
        self.src.num_imm() + self.dst.num_imm()
    }
}


////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! make_branch_ins {
    ($op:ident, $offset:expr) => { 
        Ins::Branch(BranchIns{op: BranchOpcode::$op, target: Target::Label($offset)}) 
    };
}

#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum BranchOpcode {
    Br = 1,
    Bne,
    Beq,
    Bge,
    Blt,
    Bgt,
    Ble,

    Bpl = 128,
    Bmi,
    Bhi,
    Blos,
    Bvc,
    Bvs,
    Bcc,
    Bcs
}

#[derive(Debug, Clone)]
pub struct BranchIns {
    pub op: BranchOpcode,
    pub target: Target,
}

impl BranchIns {
    pub const OFFSET_NUM_BITS: usize = 8;
    pub const OFFSET_MASK: u16 = (1u16 << Self::OFFSET_NUM_BITS) - 1;

    pub fn num_imm(&self) -> u16 {
        0
    }
}

impl InstrVariant<BranchOpcode> for BranchIns {
    const OPCODE_BITS: usize = 8;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;


}

////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! make_jmp_ins {
    ($dst:expr) => { Ins::Jmp(JmpIns{op: JmpOpcode::Jmp, dst: $dst}) };
}
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum JmpOpcode {
    Jmp = 1,
}

#[derive(Debug, Clone)]
pub struct JmpIns {
    pub op: JmpOpcode,
    pub dst: RegArg,
}

impl JmpIns {
    pub fn num_imm(&self) -> u16 {
        self.dst.num_imm()
    }
}

impl InstrVariant<JmpOpcode> for JmpIns {
    const OPCODE_BITS: usize =  10;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}


////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! make_jsr_ins {
    ($reg:expr, $dst:expr) => { Ins::Jsr(JsrIns{op: JsrOpcode::Jsr, reg: $reg, dst: $dst}) };
}
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum JsrOpcode {
    Jsr = 4,
}

#[derive(Debug, Clone)]
pub struct JsrIns {
    pub op: JsrOpcode,
    pub reg: Reg,
    pub dst: RegArg,
}

impl JsrIns {
    pub fn num_imm(&self) -> u16 {
        self.dst.num_imm()
    }
}
impl InstrVariant<JsrOpcode> for JsrIns {
    const OPCODE_BITS: usize = 7;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! make_rts_ins {
    ($reg:expr) => { Ins::Rts(RtsIns{op: RtsOpcode::Rts, reg: $reg}) };
}
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum RtsOpcode {
    Rts = 16,
}
#[derive(Debug, Clone)]
pub struct RtsIns {
    pub op: RtsOpcode,
    pub reg: Reg,
}

impl RtsIns {
    pub fn num_imm(&self) -> u16 {
        0
    }
}

impl InstrVariant<RtsOpcode> for RtsIns {
    const OPCODE_BITS: usize = 13;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

////////////////////////////////////////////////////////////////////////////////



#[macro_export]
macro_rules! make_single_operand_ins {
    ($op:ident,  $dst:expr) => { Ins::SingleOperand(SingleOperandIns{op: SingleOperandOpcode::$op, dst: $dst}) };
}
// Also rotates, single operand byte inpub structions
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum SingleOperandOpcode {
    Swab = 3,

    Clr = 40,
    Com,
    Inc,
    Dec,
    Neg,
    Adc,
    Sbc,
    Tst,
    Ror,
    Rol,
    Asr,
    Asl,

    ClrB = 552,
    ComB,
    IncB,
    DecB,
    NegB,
    AdcB,
    SbcB,
    TstB,
    RorB,
    RolB,
    AsrB,
    AslB,
}

#[derive(Debug, Clone)]
pub struct SingleOperandIns {
    pub op: SingleOperandOpcode,
    pub dst: RegArg,
}

impl SingleOperandIns {
    pub fn num_imm(&self) -> u16 {
        self.dst.num_imm()
    }

    pub fn is_byte(&self) -> bool {
        (self.op as u32) >= (SingleOperandOpcode::ClrB as u32)
    }
}

impl InstrVariant<SingleOperandOpcode> for SingleOperandIns {
    const OPCODE_BITS: usize = 10;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

////////////////////////////////////////////////////////////////////////////////


#[macro_export]
macro_rules! make_cc_ins {
    ($op:ident) => { Ins::CC(CCIns{op: CCOpcode::$op}) };
}
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum CCOpcode {
    Nop = 0o240,
    Clc = 0o241,
    Clv = 0o242,
    Clz = 0o244,
    Cln = 0o250,
    // 0o260 is an alternate nop
    Sec = 0o261,
    Sev = 0o262,
    Sez = 0o264,
    Sen = 0o270,
}


#[derive(Debug, Clone)]
pub struct CCIns {
    pub op: CCOpcode,
}

impl CCIns {
    pub fn num_imm(&self) -> u16 {
        0
    }
}

impl InstrVariant<CCOpcode> for CCIns {
    const OPCODE_BITS: usize = 16;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;

}


////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! make_misc_ins {
    ($op:ident) => { Ins::Misc(MiscIns{op: MiscOpcode::$op}) };
}
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum MiscOpcode {
    Halt = 0,
    Wait,

    // Actually part of traps, but 16 bits
    Rti = 2, // Return from interrupt
    Iox,     // I/O executive routine, no defined mnemonic
    Iot, 

    // Back to Misc proper
    Reset = 5,

}


#[derive(Debug, Clone)]
pub struct MiscIns {
    pub op: MiscOpcode,
}

impl MiscIns {
    pub fn num_imm(&self) -> u16 {
        0
    }
}

impl InstrVariant<MiscOpcode> for MiscIns {
    const OPCODE_BITS: usize = 16;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! make_trap_ins {
    ($op:ident, $handler:expr) => { TrapIns{op: MiscOpcode::$op, handler:$handler } };
}
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum TrapOpcode {
    Emt = 136,
    Trap,
}


#[derive(Debug, Clone)]
pub struct TrapIns {
    pub op: TrapOpcode,
    pub handler: Target,
}

impl TrapIns {
    pub const HANDLER_MASK: u16 = (1u16 << Self::OPCODE_BITS) - 1;

    pub fn num_imm(&self) -> u16 {
        0
    }
}

impl InstrVariant<TrapOpcode> for TrapIns {
    const OPCODE_BITS: usize = 8;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

////////////////////////////////////////////////////////////////////////////////


#[derive(Debug, Clone)]
pub enum Ins {
    DoubleOperand(DoubleOperandIns),
    Branch(BranchIns),
    Jmp(JmpIns),
    Jsr(JsrIns),
    Rts(RtsIns),
    SingleOperand(SingleOperandIns),
    CC(CCIns),
    Misc(MiscIns),
    Trap(TrapIns),
}

impl Ins {
    pub fn num_imm(&self) -> u16 {
        match self {
            Ins::DoubleOperand(x) => x.num_imm(),
            Ins::Branch(x) => x.num_imm(),
            Ins::Jmp(x) => x.num_imm(),
            Ins::Jsr(x) => x.num_imm(),
            Ins::Rts(x) => x.num_imm(),
            Ins::SingleOperand(x) => x.num_imm(),
            Ins::CC(x) => x.num_imm(),
            Ins::Misc(x) => x.num_imm(),
            Ins::Trap(x) => x.num_imm(),
        }
    }

    pub fn size(&self) -> u16 {
        WORD_SIZE + WORD_SIZE * self.num_imm()
    }

}


