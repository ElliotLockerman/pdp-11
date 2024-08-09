
use crate::constants::WORD_SIZE;

use std::fmt;

use num_derive::{FromPrimitive, ToPrimitive};    
use num_traits::{FromPrimitive, ToPrimitive};    
use derive_more::{IsVariant, Unwrap};


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

#[derive(Debug, Clone, IsVariant, Unwrap)]
pub enum Op {
    Add,
    Sub,
    And,
    Or,
}


#[derive(Debug, Clone, IsVariant, Unwrap)]
pub enum Atom {
    Loc,
    Val(u16),
    SymbolRef(String),
}

#[derive(Debug, Clone, IsVariant, Unwrap)]
pub enum Expr {
    Atom(Atom),
    Op(Box<Expr>, Op, Atom),
}

impl Expr {
    pub fn unwrap_val(&self) -> u16 {
        let Expr::Atom(atom) = &self else {
            panic!("unwrap_val() called on non-Atom Expr");
        };

        let Atom::Val(val) = atom else {
            panic!("unwrap_val() called on non-Val Expr::Atom");
        };
        *val
    }
}

#[derive(Debug, Clone, IsVariant, Unwrap)]
pub enum Extra {
    None,
    Imm(Expr),
    Rel(Expr),
}

impl Extra {
    pub fn unwrap_val(&self) -> u16 {
        match &self {
            Extra::Imm(e) => e.unwrap_val(),
            Extra::Rel(e) => e.unwrap_val(),
            Extra::None => todo!(),
        }
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

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

pub const NUM_REGS: usize = 8;

impl Reg {
    pub const NUM_BITS: usize = 3;
    pub const MASK: u16 = (1u16 << Self::NUM_BITS) - 1;
}

#[derive(Debug, Clone)]
pub struct Operand {
    pub mode: AddrMode,
    pub reg: Reg,
    pub extra: Extra, // Only used by assembler
}

impl Operand {
    pub const NUM_BITS: usize = AddrMode::NUM_BITS + Reg::NUM_BITS;

    #[allow(dead_code)]
    pub const MASK: u16 = (1u16 << Self::NUM_BITS) - 1;

    pub fn new(mode: AddrMode, reg: Reg, extra: Extra) -> Operand {
        let ret = Operand{mode, reg, extra};
        assert!(!ret.needs_extra() || !ret.extra.is_none());
        ret
    }

    pub fn needs_extra(&self) -> bool {
        use AddrMode::*;
        matches!{
            (self.mode, self.reg),
            (AutoInc | AutoIncDef, Reg::PC)
                | (Index | IndexDef, Reg::PC)
                | (Index | IndexDef, _)
        }
    }

    pub fn add_extra(&mut self, val: u16) {
        use AddrMode::*;
        let val = Expr::Atom(Atom::Val(val));
        match (self.mode, self.reg) {
            (AutoInc | AutoIncDef, Reg::PC) => self.extra = Extra::Imm(val),
            (Index | IndexDef, Reg::PC) => self.extra = Extra::Rel(val),
            (Index | IndexDef, _) => self.extra = Extra::Imm(val),
            _ => panic!("Operand with mode {:?} and reg {:?} doesn't need extra", self.mode, self.reg),
        }
    }

    pub fn has_extra(&self) -> bool {
        !self.extra.is_none()
    }

    pub fn num_extra(&self) -> u16 {
        self.has_extra() as u16
    }


    pub fn format(&self) -> u16 {
        self.reg.to_u16().unwrap() | (self.mode.to_u16().unwrap() << Reg::NUM_BITS)
    }

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, mut pc: u16) -> fmt::Result {
        pc = pc.wrapping_add(2);
        use AddrMode::*;
        match (self.mode, self.reg) {
            (Index, Reg::PC) => write!(f, "{:#o}", pc.wrapping_add(self.extra.unwrap_val())),
            (IndexDef, Reg::PC) => write!(f, "@ {:#o}", pc.wrapping_add(self.extra.unwrap_val())),
            (_, _) => fmt::Display::fmt(self, f),
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use AddrMode::*;
        match (self.mode, self.reg) {
            (Index, Reg::PC) => write!(f, ". + {:#o}", 2u16.wrapping_add(self.extra.unwrap_val())),
            (IndexDef, Reg::PC) => write!(f, "@ . + {:#o}", 2u16.wrapping_add(self.extra.unwrap_val())),
            (AutoInc, Reg::PC) => write!(f, "#{:#o}", self.extra.unwrap_val()),
            (AutoIncDef, Reg::PC) => write!(f, "@#{:#o}", self.extra.unwrap_val()),

            (Gen, _) => write!(f, "{}", self.reg),
            (Def, _) => write!(f, "({})", self.reg),
            (AutoInc, _) => write!(f, "({})+", self.reg),
            (AutoIncDef, _) => write!(f, "@({})+", self.reg),
            (AutoDec, _) => write!(f, "-({})", self.reg),
            (AutoDecDef, _) => write!(f, "@-({})", self.reg),
            (Index, _) => write!(f, "{:#o}({})", self.extra.unwrap_val(), self.reg),
            (IndexDef, _) => write!(f, "@{:#o}({})", self.extra.unwrap_val(), self.reg),
        }
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

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, mut pc: u16) -> fmt::Result {
        pc = pc.wrapping_add(2);
        match self {
            Target::Label(lbl) => write!(f, "{}", lbl),
            Target::Offset(off) => write!(f, "{:#o}", pc.wrapping_add(((*off as i8 as i16) * 2) as u16)),
            
        }
    }
}

impl fmt::Display for Target {
    // TODO: resolve actual destination.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Target::Label(lbl) => write!(f, "{}", lbl),
            Target::Offset(off) => write!(f, ". + {:#o}", 2u16.wrapping_add(((*off as i8 as i16) * 2) as u16))
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

impl fmt::Display for DoubleOperandOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}



#[derive(Debug, Clone)]
pub struct DoubleOperandIns {
    pub op: DoubleOperandOpcode,
    pub src: Operand,
    pub dst: Operand,
}


impl InstrVariant<DoubleOperandOpcode> for DoubleOperandIns {
    const OPCODE_BITS: usize = 4;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

impl DoubleOperandIns {
    pub fn num_extra(&self) -> u16 {
        self.src.num_extra() + self.dst.num_extra()
    }

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, pc: u16) -> fmt::Result {
        write!(f, "{}\t", self.op)?;
        self.src.fmt_with_pc(f, pc)?;
        write!(f, ", ")?;
        self.dst.fmt_with_pc(f, pc)
    }
}

impl fmt::Display for DoubleOperandIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}, {}", self.op, self.src, self.dst)
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

impl fmt::Display for BranchOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct BranchIns {
    pub op: BranchOpcode,
    pub target: Target,
}

impl BranchIns {
    pub const OFFSET_NUM_BITS: usize = 8;
    pub const OFFSET_MASK: u16 = (1u16 << Self::OFFSET_NUM_BITS) - 1;

    pub fn num_extra(&self) -> u16 {
        0
    }

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, pc: u16) -> fmt::Result {
        write!(f, "{}\t", self.op)?;
        self.target.fmt_with_pc(f, pc)
    }
}

impl InstrVariant<BranchOpcode> for BranchIns {
    const OPCODE_BITS: usize = 8;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

impl fmt::Display for BranchIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.op, self.target)
    }
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

impl fmt::Display for JmpOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct JmpIns {
    pub op: JmpOpcode,
    pub dst: Operand,
}

impl JmpIns {
    pub fn num_extra(&self) -> u16 {
        self.dst.num_extra()
    }

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, pc: u16) -> fmt::Result {
        write!(f, "{}\t", self.op)?;
        self.dst.fmt_with_pc(f, pc)
    }
}

impl InstrVariant<JmpOpcode> for JmpIns {
    const OPCODE_BITS: usize =  10;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

impl fmt::Display for JmpIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.op, self.dst)
    }
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

impl fmt::Display for JsrOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct JsrIns {
    pub op: JsrOpcode,
    pub reg: Reg,
    pub dst: Operand,
}

impl JsrIns {
    pub fn num_extra(&self) -> u16 {
        self.dst.num_extra()
    }

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, pc: u16) -> fmt::Result {
        write!(f, "{}\t{},", self.op, self.reg)?;
        self.dst.fmt_with_pc(f, pc)
    }
}

impl InstrVariant<JsrOpcode> for JsrIns {
    const OPCODE_BITS: usize = 7;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

impl fmt::Display for JsrIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}, {}", self.op, self.reg, self.dst)
    }
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

impl fmt::Display for RtsOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct RtsIns {
    pub op: RtsOpcode,
    pub reg: Reg,
}

impl RtsIns {
    pub fn num_extra(&self) -> u16 {
        0
    }

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, _pc: u16) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl InstrVariant<RtsOpcode> for RtsIns {
    const OPCODE_BITS: usize = 13;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

impl fmt::Display for RtsIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.op, self.reg)
    }
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

impl fmt::Display for SingleOperandOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct SingleOperandIns {
    pub op: SingleOperandOpcode,
    pub dst: Operand,
}

impl SingleOperandIns {
    pub fn num_extra(&self) -> u16 {
        self.dst.num_extra()
    }

    pub fn is_byte(&self) -> bool {
        (self.op as u32) >= (SingleOperandOpcode::ClrB as u32)
    }

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, pc: u16) -> fmt::Result {
        write!(f, "{}\t", self.op)?;
        self.dst.fmt_with_pc(f, pc)
    }
}

impl InstrVariant<SingleOperandOpcode> for SingleOperandIns {
    const OPCODE_BITS: usize = 10;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

impl fmt::Display for SingleOperandIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.op, self.dst)
    }
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

impl fmt::Display for CCOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct CCIns {
    pub op: CCOpcode,
}

impl CCIns {
    pub fn num_extra(&self) -> u16 {
        0
    }

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, _pc: u16) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl InstrVariant<CCOpcode> for CCIns {
    const OPCODE_BITS: usize = 16;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

impl fmt::Display for CCIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.op)
    }
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

impl fmt::Display for MiscOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}


#[derive(Debug, Clone)]
pub struct MiscIns {
    pub op: MiscOpcode,
}

impl MiscIns {
    pub fn num_extra(&self) -> u16 {
        0
    }

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, _pc: u16) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl InstrVariant<MiscOpcode> for MiscIns {
    const OPCODE_BITS: usize = 16;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

impl fmt::Display for MiscIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.op)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! make_trap_ins {
    ($op:ident, $data:expr) => { Ins::Trap(TrapIns{op: TrapOpcode::$op, data:$data }) };
}
#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum TrapOpcode {
    Emt = 136,
    Trap,
}

impl fmt::Display for TrapOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct TrapIns {
    pub op: TrapOpcode,
    pub data: Expr,
}

impl TrapIns {
    pub const DATA_MASK: u16 = (1u16 << Self::OPCODE_BITS) - 1;

    pub fn num_extra(&self) -> u16 {
        0
    }

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, _pc: u16) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl InstrVariant<TrapOpcode> for TrapIns {
    const OPCODE_BITS: usize = 8;
    const LOWER_BITS: usize = 16 - Self::OPCODE_BITS;
}

impl fmt::Display for TrapIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{:#o}", self.op, self.data.unwrap_val())
    }
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
    pub fn num_extra(&self) -> u16 {
        match self {
            Ins::DoubleOperand(x) => x.num_extra(),
            Ins::Branch(x) => x.num_extra(),
            Ins::Jmp(x) => x.num_extra(),
            Ins::Jsr(x) => x.num_extra(),
            Ins::Rts(x) => x.num_extra(),
            Ins::SingleOperand(x) => x.num_extra(),
            Ins::CC(x) => x.num_extra(),
            Ins::Misc(x) => x.num_extra(),
            Ins::Trap(x) => x.num_extra(),
        }
    }

    pub fn size(&self) -> u16 {
        WORD_SIZE + WORD_SIZE * self.num_extra()
    }

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, pc: u16) -> fmt::Result {
        match self {
            Ins::DoubleOperand(x) => x.fmt_with_pc(f, pc),
            Ins::Branch(x) => x.fmt_with_pc(f, pc),
            Ins::Jmp(x) => x.fmt_with_pc(f, pc),
            Ins::Jsr(x) => x.fmt_with_pc(f, pc),
            Ins::Rts(x) => x.fmt_with_pc(f, pc),
            Ins::SingleOperand(x) => x.fmt_with_pc(f, pc),
            Ins::CC(x) => x.fmt_with_pc(f, pc),
            Ins::Misc(x) => x.fmt_with_pc(f, pc),
            Ins::Trap(x) => x.fmt_with_pc(f, pc),
        }
    }

    pub fn display_with_pc(&self, pc: u16) -> InsWithPc {
        InsWithPc(self, pc)
    }
}

impl fmt::Display for Ins {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ins::DoubleOperand(ins) => write!(f, "{ins}"),
            Ins::Branch(ins) => write!(f, "{ins}"),
            Ins::Jmp(ins) => write!(f, "{ins}"),
            Ins::Jsr(ins) => write!(f, "{ins}"),
            Ins::Rts(ins) => write!(f, "{ins}"),
            Ins::SingleOperand(ins) => write!(f, "{ins}"),
            Ins::CC(ins) => write!(f, "{ins}"),
            Ins::Misc(ins) => write!(f, "{ins}"),
            Ins::Trap(ins) => write!(f, "{ins}"),
        }
    }
}

// Just for formatting, like Path::Display()
pub struct InsWithPc<'a>(&'a Ins, u16);

impl<'a> fmt::Display for InsWithPc<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt_with_pc(f, self.1)
    }
}

