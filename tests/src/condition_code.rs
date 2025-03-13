use crate::flags::{C, N, V, Z, check_flags};
use as_lib::assemble_raw;
use common::asm::Reg;
use common::constants::DATA_START;
use common::mem::ToU16P;
use emu_lib::Emulator;

fn run(ins: &str, flags_init: u16, flags_exp: u16) {
    let asm = format!(
        r#"
        {ins}
        halt
    "#
    );
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.get_state_mut().get_status_mut().set_flags(flags_init);
    emu.run_at(DATA_START);
    check_flags(&emu, flags_exp);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p()
    );
}

#[test]
fn nop() {
    run("nop", 0, 0);
    run("nop", N | C | V | Z, N | C | V | Z);
}

#[test]
fn clc() {
    run("clc", 0, 0);
    run("clc", N | V | Z | C, N | V | Z);
    run("clc", N | V | Z, N | V | Z);
}

#[test]
fn clv() {
    run("clv", 0, 0);
    run("clv", N | V | Z | C, N | C | Z);
    run("clv", N | C | Z, N | C | Z);
}

#[test]
fn clz() {
    run("clz", 0, 0);
    run("clz", N | V | Z | C, N | C | V);
    run("clz", N | C | V, N | C | V);
}

#[test]
fn cln() {
    run("cln", 0, 0);
    run("cln", N | V | Z | C, Z | C | V);
    run("cln", Z | C | V, Z | C | V);
}

#[test]
fn sec() {
    run("sec", 0, C);
    run("sec", N | V | Z, N | V | Z | C);
    run("sec", N | V | Z | C, N | V | Z | C);
}

#[test]
fn sev() {
    run("sev", 0, V);
    run("sev", N | C | Z, N | V | Z | C);
    run("sev", N | V | Z | C, N | V | Z | C);
}

#[test]
fn sez() {
    run("sez", 0, Z);
    run("sez", N | C | V, N | V | Z | C);
    run("sez", N | V | Z | C, N | V | Z | C);
}

#[test]
fn sen() {
    run("sen", 0, N);
    run("sen", Z | C | V, N | V | Z | C);
    run("sen", N | V | Z | C, N | V | Z | C);
}
