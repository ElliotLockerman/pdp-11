use as_lib::assemble;
use emu_lib::Emulator;
use common::asm::Reg;
use common::constants::DATA_START;
use crate::flags::{Flags, flags};

// Because each test is run on a fresh emulator, unaffected flags will be false
fn run(
    ins: &str,
    r0_init: u16,
    r1_init: u16,
    r1_exp: u16,
    flags_exp: Flags,
) {
    let asm = format!(r#"
        {ins} r0, r1
        halt
    "#);
    let bin = assemble(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.get_state_mut().reg_write_word(Reg::R0, r0_init);
    emu.get_state_mut().reg_write_word(Reg::R1, r1_init);
    emu.run_at(DATA_START);
    assert_eq!(emu.get_state().reg_read_word(Reg::R0), r0_init);
    assert_eq!(emu.get_state().reg_read_word(Reg::R1), r1_exp);
    let status = emu.get_state().get_status();
    assert_eq!(status.get_carry(), flags_exp.c, "carry flag");
    assert_eq!(status.get_overflow(),flags_exp.v, "overflow flag");
    assert_eq!(status.get_zero(), flags_exp.z, "zero flag");
    assert_eq!(status.get_negative(), flags_exp.n, "negative flag");
    assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}


#[test]
fn mov() {
    run("mov", 0, 0, 0, flags().z());
    run("mov", 1, 0, 1, flags());
    run("mov", 0o177777, 0, 0o177777, flags().n());
}

#[test]
fn add() {
    run("add", 0, 0, 0, flags().z());
    run("add", 0, 1, 1, flags());
    run("add", 1, 0, 1, flags());
    run("add", 1, 1, 2, flags());
    run("add", 0o177777, 0, 0o177777, flags().n());
    run("add", 1, 0o177777, 0, flags().z().c());
    run("add", 1, 0o077777, 0o100000, flags().n().v());
}

#[test]
fn sub() {
    run("sub", 0, 0, 0, flags().z());
    run("sub", 1, 1, 0, flags().z());
    run("sub", 1, 0o100000, 0o077777, flags().v());
    run("sub", 1, 0, 0o177777, flags().n().c());
    run("sub", 1, 0o177777, 0o177776, flags().n());
}

#[test]
fn cmp() {
    run("cmp", 0, 0, 0, flags().z());
    run("cmp", 1, 1, 1, flags().z());
    run("cmp", 0o100000, 1, 1, flags().v());
    run("cmp", 0, 1, 1, flags().n().c());
    run("cmp", 0o177777, 1, 1, flags().n());
}

#[test]
fn bis() {
    run("bis", 0o0, 0o0, 0o0, flags().z());
    run("bis", 0o1, 0o1, 0o1, flags());
    run("bis", 0o1, 0o2, 0o3, flags());
    run("bis", 0o170000, 0o20, 0o170020, flags().n());
}

#[test]
fn bic() {
    run("bic", 0o0, 0o0, 0o0, flags().z());
    run("bic", 0o7, 0o7, 0o0, flags().z());
    run("bic", 0o2, 0o7, 0o5, flags());
    run("bic", 0o0, 0o177777, 0o177777, flags().n());
    run("bic", 0o1, 0o177777, 0o177776, flags().n());
}

#[test]
fn bit() {
    run("bit", 0o0, 0o0, 0o0, flags().z());
    run("bit", 0o1, 0o1, 0o1, flags());
    run("bit", 0o1, 0o2, 0o2, flags().z());
    run("bit", 0o100000, 0o100000, 0o100000, flags().n());
}

#[test]
fn movb() {
    run("movb", 0, 0, 0, flags().z());
    run("movb", 1, 0, 1, flags());
    run("movb", 0o377, 0, 0o177777, flags().n());
    run("movb", 0o400, 0, 0, flags().z());
}

#[test]
fn cmpb() {
    run("cmpb", 0, 0, 0, flags().z());
    run("cmpb", 1, 1, 1, flags().z());
    run("cmpb", 0o200, 1, 1, flags().v());
    run("cmpb", 0, 1, 1, flags().n().c());
    run("cmpb", 0o277, 1, 1, flags().n());

    run("cmpb", 0o30000, 0o3000, 0o3000, flags().z());
    run("cmpb", 1, 1, 1, flags().z());
    run("cmpb", 0o32200, 1, 1, flags().v());
    run("cmpb", 0o2000, 0o23001, 0o23001, flags().n().c());
    run("cmpb", 0o7277, 1, 1, flags().n());
}

#[test]
fn bisb() {
    run("bisb", 0o0, 0o0, 0o0, flags().z());
    run("bisb", 0o1, 0o1, 0o1, flags());
    run("bisb", 0o1, 0o2, 0o3, flags());
    run("bisb", 0o207, 0o20, 0o227, flags().n());
    run("bisb", 0o207, 0o7020, 0o7227, flags().n());
    run("bisb", 0o50207, 0o7020, 0o7227, flags().n());
}

#[test]
fn bicb() {
    run("bicb", 0o0, 0o0, 0o0, flags().z());
    run("bicb", 0o7, 0o7, 0o0, flags().z());
    run("bicb", 0o2, 0o7, 0o5, flags());
    run("bicb", 0o10000, 0o177777, 0o177777, flags().n());
    run("bicb", 0o10000, 0o77, 0o77, flags());
    run("bicb", 0o10010, 0o177777, 0o177767, flags().n());
    run("bicb", 0o10010, 0o177477, 0o177467, flags());
}

#[test]
fn bitb() {
    run("bitb", 0o0, 0o0, 0o0, flags().z());
    run("bitb", 0o1, 0o1, 0o1, flags());
    run("bitb", 0o1, 0o2, 0o2, flags().z());
    run("bitb", 0o200, 0o200, 0o200, flags().n());
    run("bitb", 0o100000, 0o100000, 0o100000, flags().z());
}

