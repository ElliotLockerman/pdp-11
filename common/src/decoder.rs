
use num_traits::FromPrimitive;

use super::asm::*;

fn decode_operand(arg: u16, input: &[u16], imm_idx: usize) -> Operand {
    let reg = Reg::from_u16(arg & Reg::MASK).unwrap();
    let mode = AddrMode::from_u16((arg >> Reg::NUM_BITS) & AddrMode::MASK).unwrap();

    let mut op = Operand{mode, reg, extra: Extra::None};
    if op.needs_extra() {
        op.add_extra(input[imm_idx]);
    }
    op
}


fn decode_double_operand_ins(input: &[u16]) -> Option<Ins> {
    let op = DoubleOperandIns::decode_opcode(input[0])?;

    let src = decode_operand(input[0] >> Operand::NUM_BITS, input, 1);
    let dst = decode_operand(input[0], input, (src.num_extra() + 1) as usize);

    Some(Ins::DoubleOperand(DoubleOperandIns{op, src, dst}))
}

fn decode_branch_ins(input: &[u16]) -> Option<Ins> {
    let op = BranchIns::decode_opcode(input[0])?;
    let offset = Target::Offset((input[0] & BranchIns::OFFSET_MASK) as u8);
    Some(Ins::Branch(BranchIns{op, target: offset}))
}

fn decode_jmp_ins(input: &[u16]) -> Option<Ins> {
    let op = JmpIns::decode_opcode(input[0])?;
    let dst = decode_operand(input[0], input, 1);
    Some(Ins::Jmp(JmpIns{op, dst}))
}

fn decode_jsr_ins(input: &[u16]) -> Option<Ins> {
    let op = JsrIns::decode_opcode(input[0])?;
    let dst = decode_operand(input[0], input, 1);
    let reg = Reg::from_u16((input[0] >> Operand::NUM_BITS) & Reg::MASK).unwrap();
    Some(Ins::Jsr(JsrIns{op, reg, dst}))
}

fn decode_rts_ins(input: &[u16]) -> Option<Ins> {
    let op = RtsIns::decode_opcode(input[0])?;
    let reg = Reg::from_u16(input[0] & Reg::MASK).unwrap();
    Some(Ins::Rts(RtsIns{op, reg}))
}

fn decode_single_operand_ins(input: &[u16]) -> Option<Ins> {
    let op = SingleOperandIns::decode_opcode(input[0])?;
    let dst = decode_operand(input[0], input, 1);
    Some(Ins::SingleOperand(SingleOperandIns{op, dst}))
}

fn decode_cc_ins(input: &[u16]) -> Option<Ins> {
    let op = CCIns::decode_opcode(input[0])?;
    Some(Ins::CC(CCIns{op}))
}

fn decode_misc_ins(input: &[u16]) -> Option<Ins> {
    let op = MiscIns::decode_opcode(input[0])?;
    Some(Ins::Misc(MiscIns{op}))
}

fn decode_trap_ins(input: &[u16]) -> Option<Ins> {
    let op = TrapIns::decode_opcode(input[0])?;
    let data = input[0] & TrapIns::DATA_MASK;
    Some(Ins::Trap(TrapIns{op, data: Expr::Atom(Atom::Val(data))}))
}

type Decoder = fn(&[u16]) -> Option<Ins>;

const DECODERS: &[Decoder] = &[
    decode_double_operand_ins,
    decode_branch_ins,
    decode_jmp_ins,
    decode_jsr_ins,
    decode_rts_ins,
    decode_single_operand_ins,
    decode_cc_ins,
    decode_misc_ins,
    decode_trap_ins,
]; 


pub fn decode(input: &[u16]) -> Option<Ins> {
    for decoder in DECODERS {
        let ins = decoder(input);
        if ins.is_some() {
            return ins;
        }
    }

    None
}

