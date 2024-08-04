use as_lib::assemble;
use emu_lib::Emulator;
use common::asm::Reg;
use common::constants::DATA_START;
use crate::flags::{flags, Flags};

fn run(
    ins: &str,
    flags_init: Flags,
    flags_exp: Flags,
) {
    let asm = format!(r#"
        {ins}
        halt
    "#);
    let bin = assemble(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.get_state_mut().get_status_mut().set_flags(flags_init.to_bits());
    emu.run_at(DATA_START);
    let status = emu.get_state().get_status();
    assert_eq!(status.get_carry(), flags_exp.c, "carry flag");
    assert_eq!(status.get_overflow(),flags_exp.v, "overflow flag");
    assert_eq!(status.get_zero(), flags_exp.z, "zero flag");
    assert_eq!(status.get_negative(), flags_exp.n, "negative flag");
    assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn nop() {
    run("nop", flags(), flags());
    run("nop", flags().n().c().v().z(), flags().n().c().v().z());
}

#[test]
fn clc() {
    run("clc", flags(), flags());
    run("clc", flags().n().v().z().c(), flags().n().v().z());
    run("clc", flags().n().v().z(), flags().n().v().z());
}

#[test]
fn clv() {
    run("clv", flags(), flags());
    run("clv", flags().n().v().z().c(), flags().n().c().z());
    run("clv", flags().n().c().z(), flags().n().c().z());
}

#[test]
fn clz() {
    run("clz", flags(), flags());
    run("clz", flags().n().v().z().c(), flags().n().c().v());
    run("clz", flags().n().c().v(), flags().n().c().v());
}

#[test]
fn cln() {
    run("cln", flags(), flags());
    run("cln", flags().n().v().z().c(), flags().z().c().v());
    run("cln", flags().z().c().v(), flags().z().c().v());
}

#[test]
fn sec() {
    run("sec", flags(), flags().c());
    run("sec", flags().n().v().z(), flags().n().v().z().c());
    run("sec", flags().n().v().z().c(), flags().n().v().z().c());
}

#[test]
fn sev() {
    run("sev", flags(), flags().v());
    run("sev", flags().n().c().z(), flags().n().v().z().c());
    run("sev", flags().n().v().z().c(), flags().n().v().z().c());
}

#[test]
fn sez() {
    run("sez", flags(), flags().z());
    run("sez", flags().n().c().v(), flags().n().v().z().c());
    run("sez", flags().n().v().z().c(), flags().n().v().z().c());
}

#[test]
fn sen() {
    run("sen", flags(), flags().n());
    run("sen", flags().z().c().v(), flags().n().v().z().c());
    run("sen", flags().n().v().z().c(), flags().n().v().z().c());
}


