
use crate::constants::WORD_SIZE;
use crate::mem::write_u16;

use std::fmt;
use std::io::Write;

use num_derive::{FromPrimitive, ToPrimitive};    
use num_traits::{FromPrimitive, ToPrimitive};    
use derive_more::{IsVariant, Unwrap};
use delegate::delegate;


pub trait InstrVariant<Opcode: FromPrimitive> {
    const OPCODE_BITS: usize;
    const LOWER_BITS: usize = (u16::BITS as usize) - Self::OPCODE_BITS;

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

impl Op {
    pub fn to_char(&self) -> char {
        use Op::*;
        match self {
            Add => '+',
            Sub => '-',
            And => '&',
            Or => '!',
        }
    }
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


    pub fn encode(&self) -> u16 {
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

    fn decode(arg: u16, input: &[u16], imm_idx: usize) -> Operand {
        let reg = Reg::from_u16(arg & Reg::MASK).unwrap();
        let mode = AddrMode::from_u16((arg >> Reg::NUM_BITS) & AddrMode::MASK).unwrap();

        let mut op = Self{mode, reg, extra: Extra::None};
        if op.needs_extra() {
            op.add_extra(input[imm_idx]);
        }
        op
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
macro_rules! double_operand_ins {
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

    pub fn emit(&self, out: &mut impl Write) {
        let bin = (self.op.to_u16().unwrap() << Self::LOWER_BITS) 
            | (self.src.encode() << Operand::NUM_BITS) 
            | self.dst.encode();
        write_u16(out, bin);

        if self.src.has_extra() {
            write_u16(out, self.src.extra.unwrap_val());
        }

        if self.dst.has_extra() {
            write_u16(out, self.dst.extra.unwrap_val());
        }
    }

    fn decode(input: &[u16]) -> Option<Ins> {
        let op = DoubleOperandIns::decode_opcode(input[0])?;

        let src = Operand::decode(input[0] >> Operand::NUM_BITS, input, 1);
        let dst = Operand::decode(input[0], input, (src.num_extra() + 1) as usize);

        Some(Ins::DoubleOperand(Self{op, src, dst}))
    }
}

impl fmt::Display for DoubleOperandIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}, {}", self.op, self.src, self.dst)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! branch_ins {
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

    pub fn emit(&self, out: &mut impl Write) {
        let offset = self.target.unwrap_offset();
        let bin = (self.op.to_u16().unwrap() << Self::LOWER_BITS) | (offset as u16);
        write_u16(out, bin);
    }

    fn decode(input: &[u16]) -> Option<Ins> {
        let op = Self::decode_opcode(input[0])?;
        let offset = Target::Offset((input[0] & Self::OFFSET_MASK) as u8);
        Some(Ins::Branch(Self{op, target: offset}))
    }
}

impl InstrVariant<BranchOpcode> for BranchIns {
    const OPCODE_BITS: usize = 8;
}

impl fmt::Display for BranchIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.op, self.target)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! jmp_ins {
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

    pub fn emit(&self, out: &mut impl Write) {
        let bin = (self.op.to_u16().unwrap() << Self::LOWER_BITS) | self.dst.encode();
        write_u16(out, bin);
        if self.dst.has_extra() {
            write_u16(out, self.dst.extra.unwrap_val());
        }
    }

    fn decode(input: &[u16]) -> Option<Ins> {
        let op = Self::decode_opcode(input[0])?;
        let dst = Operand::decode(input[0], input, 1);
        Some(Ins::Jmp(Self{op, dst}))
    }
}

impl InstrVariant<JmpOpcode> for JmpIns {
    const OPCODE_BITS: usize =  10;
}

impl fmt::Display for JmpIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.op, self.dst)
    }
}


////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! jsr_ins {
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

    pub fn emit(&self, out: &mut impl Write) {
        let bin = (self.op.to_u16().unwrap() << Self::LOWER_BITS)
            | (self.reg.to_u16().unwrap() << Operand::NUM_BITS)
            | self.dst.encode();
        write_u16(out, bin);
        if self.dst.has_extra() {
            write_u16(out, self.dst.extra.unwrap_val());
        }
    }
    fn decode(input: &[u16]) -> Option<Ins> {
        let op = Self::decode_opcode(input[0])?;
        let dst = Operand::decode(input[0], input, 1);
        let reg = Reg::from_u16((input[0] >> Operand::NUM_BITS) & Reg::MASK).unwrap();
        Some(Ins::Jsr(Self{op, reg, dst}))
    }
}

impl InstrVariant<JsrOpcode> for JsrIns {
    const OPCODE_BITS: usize = 7;
}

impl fmt::Display for JsrIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}, {}", self.op, self.reg, self.dst)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! rts_ins {
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

    pub fn emit(&self, out: &mut impl Write) {
        let bin = (self.op.to_u16().unwrap() << Self::LOWER_BITS) | self.reg.to_u16().unwrap();
        write_u16(out, bin);
    }

    fn decode(input: &[u16]) -> Option<Ins> {
        let op = Self::decode_opcode(input[0])?;
        let reg = Reg::from_u16(input[0] & Reg::MASK).unwrap();
        Some(Ins::Rts(Self{op, reg}))
    }
}

impl InstrVariant<RtsOpcode> for RtsIns {
    const OPCODE_BITS: usize = 13;
}

impl fmt::Display for RtsIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.op, self.reg)
    }
}

////////////////////////////////////////////////////////////////////////////////



#[macro_export]
macro_rules! single_operand_ins {
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

    pub fn emit(&self, out: &mut impl Write) {
        let bin = (self.op.to_u16().unwrap() << Self::LOWER_BITS) | self.dst.encode();
        write_u16(out, bin);
        if self.dst.has_extra() {
            write_u16(out, self.dst.extra.unwrap_val());
        }

    }

    fn decode(input: &[u16]) -> Option<Ins> {
        let op = Self::decode_opcode(input[0])?;
        let dst = Operand::decode(input[0], input, 1);
        Some(Ins::SingleOperand(Self{op, dst}))
    }
}

impl InstrVariant<SingleOperandOpcode> for SingleOperandIns {
    const OPCODE_BITS: usize = 10;
}

impl fmt::Display for SingleOperandIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}", self.op, self.dst)
    }
}

////////////////////////////////////////////////////////////////////////////////

// KE11-E Extended Instruction Set instructions

#[macro_export]
macro_rules! eis_ins {
    ($op:ident,  $reg:expr, $dst:expr) => { Ins::Eis(EisIns{op: EisOpcode::$op, reg: $reg, operand: $dst}) };
}

#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive, PartialEq, Eq)]
pub enum EisOpcode {
    Mul = 0o70,
    Div,
    Ash,
    Ashc,
    Xor, // Not technically an EIS instruction, but it has the same format, except operand is dst.
}

impl fmt::Display for EisOpcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct EisIns {
    pub op: EisOpcode,
    pub reg: Reg,
    pub operand: Operand, // src, except for Xor, where its dst.
}

impl EisIns {
    pub fn num_extra(&self) -> u16 {
        self.operand.num_extra()
    }

    pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, pc: u16) -> fmt::Result {
        write!(f, "{}\t{},", self.op, self.reg)?;
        self.operand.fmt_with_pc(f, pc)
    }

    pub fn emit(&self, out: &mut impl Write) {
        let reg = self.reg.to_u16().unwrap();
        if self.op == EisOpcode::Div {
            assert_eq!(reg & 0x1, 0, "Division reg must be even");
        }
        let bin = (self.op.to_u16().unwrap() << Self::LOWER_BITS) 
            | (reg << Operand::NUM_BITS)
            | self.operand.encode();
        write_u16(out, bin);
        if self.operand.has_extra() {
            write_u16(out, self.operand.extra.unwrap_val());
        }
    }

    fn decode(input: &[u16]) -> Option<Ins> {
        let op = Self::decode_opcode(input[0])?;
        let operand = Operand::decode(input[0], input, 1);
        let reg = Reg::from_u16((input[0] >> Operand::NUM_BITS) & Reg::MASK).unwrap();
        Some(Ins::Eis(Self{op, reg, operand}))
    }
}

impl InstrVariant<EisOpcode> for EisIns {
    const OPCODE_BITS: usize = 7;
}

impl fmt::Display for EisIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}, {}", self.op, self.reg, self.operand)
    }
}


////////////////////////////////////////////////////////////////////////////////



#[macro_export]
macro_rules! cc_ins {
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

    pub fn emit(&self, out: &mut impl Write) {
        write_u16(out, self.op.to_u16().unwrap());
    }

    fn decode(input: &[u16]) -> Option<Ins> {
        let op = Self::decode_opcode(input[0])?;
        Some(Ins::CC(Self{op}))
    }
}

impl InstrVariant<CCOpcode> for CCIns {
    const OPCODE_BITS: usize = 16;
}

impl fmt::Display for CCIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.op)
    }
}


////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! misc_ins {
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

    pub fn emit(&self, out: &mut impl Write) {
        write_u16(out, self.op.to_u16().unwrap());
    }

    fn decode(input: &[u16]) -> Option<Ins> {
        let op = Self::decode_opcode(input[0])?;
        Some(Ins::Misc(Self{op}))
    }
}

impl InstrVariant<MiscOpcode> for MiscIns {
    const OPCODE_BITS: usize = 16;
}

impl fmt::Display for MiscIns {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.op)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[macro_export]
macro_rules! trap_ins {
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

    pub fn emit(&self, out: &mut impl Write) {
        let data = self.data.unwrap_val();
        assert_eq!(data & !0xff, 0);
        write_u16(out, (self.op.to_u16().unwrap() << Self::LOWER_BITS) | data);
    }

    fn decode(input: &[u16]) -> Option<Ins> {
        let op = Self::decode_opcode(input[0])?;
        let data = input[0] & Self::DATA_MASK;
        Some(Ins::Trap(Self{op, data: Expr::Atom(Atom::Val(data))}))
    }
}

impl InstrVariant<TrapOpcode> for TrapIns {
    const OPCODE_BITS: usize = 8;
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
    Eis(EisIns),
    CC(CCIns),
    Misc(MiscIns),
    Trap(TrapIns),
}

impl Ins {
    delegate! {
        to match self {
            Ins::DoubleOperand(x) => x,
            Ins::Branch(x) => x,
            Ins::Jmp(x) => x,
            Ins::Jsr(x) => x,
            Ins::Rts(x) => x,
            Ins::SingleOperand(x) => x,
            Ins::Eis(x) => x,
            Ins::CC(x) => x,
            Ins::Misc(x) => x,
            Ins::Trap(x) => x,
        } {
            pub fn num_extra(&self) -> u16;
            pub fn fmt_with_pc(&self, f: &mut fmt::Formatter, pc: u16) -> fmt::Result;
            pub fn emit(&self, out: &mut impl Write);
        }
    }

    pub fn size(&self) -> u16 {
        WORD_SIZE + WORD_SIZE * self.num_extra()
    }

    pub fn display_with_pc(&self, pc: u16) -> InsWithPc {
        InsWithPc(self, pc)
    }

    const DECODERS: &[Decoder] = &[
        DoubleOperandIns::decode,
        BranchIns::decode,
        JmpIns::decode,
        JsrIns::decode,
        RtsIns::decode,
        SingleOperandIns::decode,
        EisIns::decode,
        CCIns::decode,
        MiscIns::decode,
        TrapIns::decode,
    ]; 

    pub fn decode(input: &[u16]) -> Option<Ins> {
        for decoder in Self::DECODERS {
            let ins = decoder(input);
            if ins.is_some() {
                return ins;
            }
        }

        None
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
            Ins::Eis(ins) => write!(f, "{ins}"),
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


type Decoder = fn(&[u16]) -> Option<Ins>;

