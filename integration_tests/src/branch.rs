use as_lib::assemble;
use emu_lib::Emulator;
use common::asm::Reg;
use common::constants::DATA_START;
use crate::flags::{Flags, flags};

fn run(ins: &str, flags: Flags, should_take: bool) {

    let asm = format!(r#"
        {ins} taken

        mov #1, r0
        halt

    taken:
        mov #2, r0
        halt
    "#);

    let bin = assemble(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.get_state_mut().get_status_mut().set_flags(flags.to_bits());
    emu.run_at(DATA_START);
    let r0 = emu.get_state().reg_read_word(Reg::R0);
    let taken = match r0 {
        1 => false,
        2 => true,
        _ => panic!("Invalid r0: {r0:o}"),
    };
    assert_eq!(taken, should_take, "branch");
}

#[test]
fn test_br() {
    run("br", flags(), true);
    run("br", flags().n(), true);
    run("br", flags().z(), true);
    run("br", flags().n().z().v().c(), true);
}

#[test]
fn test_beq() {
    run("beq", flags(), false);
    run("beq", flags().n(), false);
    run("beq", flags().z(), true);
    run("beq", flags().n().z().v().c(), true);
}

#[test]
fn test_bne() {
    run("bne", flags(), true);
    run("bne", flags().n(), true);
    run("bne", flags().z(), false);
    run("bne", flags().n().z().v().c(), false);
}

#[test]
fn test_bni() {
    run("bmi", flags(), false);
    run("bmi", flags().n(), true);
    run("bmi", flags().z(), false);
    run("bmi", flags().n().z().v().c(), true);
}

#[test]
fn test_bpl() {
    run("bpl", flags(), true);
    run("bpl", flags().n(), false);
    run("bpl", flags().z(), true);
    run("bpl", flags().n().z().v().c(), false);
}

#[test]
fn test_bcs() {
    run("bcs", flags(), false);
    run("bcs", flags().n(), false);
    run("bcs", flags().c(), true);
    run("bcs", flags().n().z().v().c(), true);
}

#[test]
fn test_bcc() {
    run("bcc", flags(), true);
    run("bcc", flags().n(), true);
    run("bcc", flags().c(), false);
    run("bcc", flags().n().z().v().c(), false);
}

#[test]
fn test_bvs() {
    run("bvs", flags(), false);
    run("bvs", flags().n(), false);
    run("bvs", flags().v(), true);
    run("bvs", flags().n().z().v().c(), true);
}

#[test]
fn test_bvc() {
    run("bvc", flags(), true);
    run("bvc", flags().n(), true);
    run("bvc", flags().v(), false);
    run("bvc", flags().n().z().v().c(), false);
}

#[test]
fn test_blt() {
    run("blt", flags(), false);
    run("blt", flags().n().v(), false);
    run("blt", flags().n().z().v().c(), false);
    run("blt", flags().n(), true);
    run("blt", flags().v(), true);
    run("blt", flags().v().c(), true);
    run("blt", flags().n().z(), true);
}

#[test]
fn test_bge() {
    run("bge", flags(), true);
    run("bge", flags().n().v(), true);
    run("bge", flags().n().z().v().c(), true);
    run("bge", flags().n(), false);
    run("bge", flags().v(), false);
    run("bge", flags().v().c(), false);
    run("bge", flags().n().z(), false);
}

#[test]
fn test_ble() {
    run("ble", flags(), false);
    run("ble", flags().z(), true);
    run("ble", flags().n().v(), false);
    run("ble", flags().n().v().z(), true);
    run("ble", flags().n().z().v().c(), true);
    run("ble", flags().n(), true);
    run("ble", flags().v(), true);
    run("ble", flags().v().c(), true);
    run("ble", flags().n().z(), true);
}

#[test]
fn test_bgt() {
    run("bgt", flags(), true);
    run("bgt", flags().z(), false);
    run("bgt", flags().n().v(), true);
    run("bgt", flags().n().v().z(), false);
    run("bgt", flags().n().z().v().c(), false);
    run("bgt", flags().n(), false);
    run("bgt", flags().v(), false);
    run("bgt", flags().v().c(), false);
    run("bgt", flags().n().z(), false);
}

#[test]
fn test_bhi() {
    run("bhi", flags(), true);
    run("bhi", flags().n().v(), true);
    run("bhi", flags().c(), false);
    run("bhi", flags().z(), false);
    run("bhi", flags().z().c().v().n(), false);
}

#[test]
fn test_blos() {
    run("blos", flags(), false);
    run("blos", flags().n().v(), false);
    run("blos", flags().c(), true);
    run("blos", flags().z(), true);
    run("blos", flags().z().c().v().n(), true);
}

#[test]
fn test_bhis() {
    run("bhis", flags(), true);
    run("bhis", flags().n(), true);
    run("bhis", flags().c(), false);
    run("bhis", flags().n().z().v().c(), false);
}

#[test]
fn test_blo() {
    run("blo", flags(), false);
    run("blo", flags().n(), false);
    run("blo", flags().c(), true);
    run("blo", flags().n().z().v().c(), true);
}

