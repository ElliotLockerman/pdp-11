use crate::flags::{check_flags, C, N, V, Z};
use as_lib::assemble_raw;
use common::asm::Reg;
use common::constants::DATA_START;
use emu_lib::Emulator;

// Because each test is run on a fresh emulator, unaffected flags will be false
fn run(ins: &str, r0_init: u16, r1_init: u16, r1_exp: u16, flags_exp: u16) {
    let asm = format!(
        r#"
        {ins} r0, r1
        halt
    "#
    );
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.reg_write_word(Reg::R0, r0_init);
    emu.reg_write_word(Reg::R1, r1_init);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), r0_init);
    assert_eq!(emu.reg_read_word(Reg::R1), r1_exp);
    check_flags(&emu, flags_exp);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len() as u16
    );
}

#[test]
fn mov() {
    run("mov", 0, 0, 0, Z);
    run("mov", 1, 0, 1, 0);
    run("mov", 0o177777, 0, 0o177777, N);
}

#[test]
fn add() {
    run("add", 0, 0, 0, Z);
    run("add", 0, 1, 1, 0);
    run("add", 1, 0, 1, 0);
    run("add", 1, 1, 2, 0);
    run("add", 0o177777, 0, 0o177777, N);
    run("add", 1, 0o177777, 0, Z | C);
    run("add", 1, 0o077777, 0o100000, N | V);
}

#[test]
fn sub() {
    run("sub", 0, 0, 0, Z);
    run("sub", 1, 1, 0, Z);
    run("sub", 1, 0o100000, 0o077777, V);
    run("sub", 1, 0, 0o177777, N | C);
    run("sub", 1, 0o177777, 0o177776, N);
}

#[test]
fn cmp() {
    run("cmp", 0, 0, 0, Z);
    run("cmp", 1, 1, 1, Z);
    run("cmp", 0o100000, 1, 1, V);
    run("cmp", 0, 1, 1, N | C);
    run("cmp", 0o177777, 1, 1, N);
}

#[test]
fn bis() {
    run("bis", 0o0, 0o0, 0o0, Z);
    run("bis", 0o1, 0o1, 0o1, 0);
    run("bis", 0o1, 0o2, 0o3, 0);
    run("bis", 0o170000, 0o20, 0o170020, N);
}

#[test]
fn bic() {
    run("bic", 0o0, 0o0, 0o0, Z);
    run("bic", 0o7, 0o7, 0o0, Z);
    run("bic", 0o2, 0o7, 0o5, 0);
    run("bic", 0o0, 0o177777, 0o177777, N);
    run("bic", 0o1, 0o177777, 0o177776, N);
}

#[test]
fn bit() {
    run("bit", 0o0, 0o0, 0o0, Z);
    run("bit", 0o1, 0o1, 0o1, 0);
    run("bit", 0o1, 0o2, 0o2, Z);
    run("bit", 0o100000, 0o100000, 0o100000, N);
}

#[test]
fn movb() {
    run("movb", 0, 0, 0, Z);
    run("movb", 1, 0, 1, 0);
    run("movb", 0o377, 0, 0o177777, N);
    run("movb", 0o400, 0, 0, Z);
}

#[test]
fn cmpb() {
    run("cmpb", 0, 0, 0, Z);
    run("cmpb", 1, 1, 1, Z);
    run("cmpb", 0o200, 1, 1, V);
    run("cmpb", 0, 1, 1, N | C);
    run("cmpb", 0o277, 1, 1, N);

    run("cmpb", 0o30000, 0o3000, 0o3000, Z);
    run("cmpb", 1, 1, 1, Z);
    run("cmpb", 0o32200, 1, 1, V);
    run("cmpb", 0o2000, 0o23001, 0o23001, N | C);
    run("cmpb", 0o7277, 1, 1, N);
}

#[test]
fn bisb() {
    run("bisb", 0o0, 0o0, 0o0, Z);
    run("bisb", 0o1, 0o1, 0o1, 0);
    run("bisb", 0o1, 0o2, 0o3, 0);
    run("bisb", 0o207, 0o20, 0o227, N);
    run("bisb", 0o207, 0o7020, 0o7227, N);
    run("bisb", 0o50207, 0o7020, 0o7227, N);
}

#[test]
fn bicb() {
    run("bicb", 0o0, 0o0, 0o0, Z);
    run("bicb", 0o7, 0o7, 0o0, Z);
    run("bicb", 0o2, 0o7, 0o5, 0);
    run("bicb", 0o10000, 0o177777, 0o177777, N);
    run("bicb", 0o10000, 0o77, 0o77, 0);
    run("bicb", 0o10010, 0o177777, 0o177767, N);
    run("bicb", 0o10010, 0o177477, 0o177467, 0);
}

#[test]
fn bitb() {
    run("bitb", 0o0, 0o0, 0o0, Z);
    run("bitb", 0o1, 0o1, 0o1, 0);
    run("bitb", 0o1, 0o2, 0o2, Z);
    run("bitb", 0o200, 0o200, 0o200, N);
    run("bitb", 0o100000, 0o100000, 0o100000, Z);
}
