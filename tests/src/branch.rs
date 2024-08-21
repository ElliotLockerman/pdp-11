use as_lib::assemble;
use emu_lib::Emulator;
use common::asm::Reg;
use common::constants::DATA_START;
use crate::flags::{C, V, Z, N};

fn run(ins: &str, flags: u16, should_take: bool) {
    let asm = format!(r#"
        {ins} taken

        mov #1, r0
        halt

    taken:
        mov #2, r0
        halt
    "#);

    let prog = assemble(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.get_state_mut().get_status_mut().set_flags(flags);
    emu.run_at(DATA_START);
    let r0 = emu.reg_read_word(Reg::R0);
    let taken = match r0 {
        1 => false,
        2 => true,
        _ => panic!("Invalid r0: {r0:o}"),
    };
    assert_eq!(taken, should_take, "branch");
}

#[test]
fn br() {
    run("br", 0, true);
    run("br", N, true);
    run("br", Z, true);
    run("br", N | Z | V | C, true);
}

#[test]
fn beq() {
    run("beq", 0, false);
    run("beq", N, false);
    run("beq", Z, true);
    run("beq", N | Z | V | C, true);
}

#[test]
fn bne() {
    run("bne", 0, true);
    run("bne", N, true);
    run("bne", Z, false);
    run("bne", N | Z | V | C, false);
}

#[test]
fn bni() {
    run("bmi", 0, false);
    run("bmi", N, true);
    run("bmi", Z, false);
    run("bmi", N | Z | V | C, true);
}

#[test]
fn bpl() {
    run("bpl", 0, true);
    run("bpl", N, false);
    run("bpl", Z, true);
    run("bpl", N | Z | V | C, false);
}

#[test]
fn bcs() {
    run("bcs", 0, false);
    run("bcs", N, false);
    run("bcs", C, true);
    run("bcs", N | Z | V | C, true);
}

#[test]
fn bcc() {
    run("bcc", 0, true);
    run("bcc", N, true);
    run("bcc", C, false);
    run("bcc", N | Z | V | C, false);
}

#[test]
fn bvs() {
    run("bvs", 0, false);
    run("bvs", N, false);
    run("bvs", V, true);
    run("bvs", N | Z | V | C, true);
}

#[test]
fn bvc() {
    run("bvc", 0, true);
    run("bvc", N, true);
    run("bvc", V, false);
    run("bvc", N | Z | V | C, false);
}

#[test]
fn blt() {
    run("blt", 0, false);
    run("blt", N | V, false);
    run("blt", N | Z | V | C, false);
    run("blt", N, true);
    run("blt", V, true);
    run("blt", V | C, true);
    run("blt", N | Z, true);
}

#[test]
fn bge() {
    run("bge", 0, true);
    run("bge", N | V, true);
    run("bge", N | Z | V | C, true);
    run("bge", N, false);
    run("bge", V, false);
    run("bge", V | C, false);
    run("bge", N | Z, false);
}

#[test]
fn ble() {
    run("ble", 0, false);
    run("ble", Z, true);
    run("ble", N | V, false);
    run("ble", N | V | Z, true);
    run("ble", N | Z | V | C, true);
    run("ble", N, true);
    run("ble", V, true);
    run("ble", V | C, true);
    run("ble", N | Z, true);
}

#[test]
fn bgt() {
    run("bgt", 0, true);
    run("bgt", Z, false);
    run("bgt", N | V, true);
    run("bgt", N | V | Z, false);
    run("bgt", N | Z | V | C, false);
    run("bgt", N, false);
    run("bgt", V, false);
    run("bgt", V | C, false);
    run("bgt", N | Z, false);
}

#[test]
fn bhi() {
    run("bhi", 0, true);
    run("bhi", N | V, true);
    run("bhi", C, false);
    run("bhi", Z, false);
    run("bhi", Z | C | V | N, false);
}

#[test]
fn blos() {
    run("blos", 0, false);
    run("blos", N | V, false);
    run("blos", C, true);
    run("blos", Z, true);
    run("blos", Z | C | V | N, true);
}

#[test]
fn bhis() {
    run("bhis", 0, true);
    run("bhis", N, true);
    run("bhis", C, false);
    run("bhis", N | Z | V | C, false);
}

#[test]
fn blo() {
    run("blo", 0, false);
    run("blo", N, false);
    run("blo", C, true);
    run("blo", N | Z | V | C, true);
}

#[test]
#[should_panic]
fn far_br() {
    let asm = r#"
        . = 400

        br label

        . = . + 10000

        label:
            halt
    "#;

    assemble(&asm);
}
