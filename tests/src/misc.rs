use as_lib::{assemble, assemble_with_symbols};
use emu_lib::Emulator;
use common::asm::Reg;
use common::constants::DATA_START;

#[test]
#[should_panic]
fn unaligned_a() {
    let bin = assemble(r#"
        jmp start

        . = 11

    start:
        mov #1, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
}

#[test]
#[should_panic]
fn unaligned_b() {
    let bin = assemble(r#"
        jmp start

        .byte 0

    start:
        mov #1, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
}

#[test]
fn even() {
    let bin = assemble(r#"
        jmp start

        . = 10

        .even
    start:
        mov #1, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 1);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        jmp start

        . = 11

        .even
    start:
        mov #1, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 1);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        jmp start

        .word 0

        .even
    start:
        mov #1, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 1);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        jmp start

        .byte 0

        .even
    start:
        mov #1, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 1);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn cont() {
    let asm = r#"
        . = 400
    _start:
        halt
        mov #1, r0
        halt
        mov #2, r0
        halt
    "#;
    let (bin, symbols) = assemble_with_symbols(asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, 0);
    emu.run_at(*symbols.get("_start").unwrap());
    assert_eq!(emu.reg_read_word(Reg::R0), 0);
    emu.cont();
    assert_eq!(emu.reg_read_word(Reg::R0), 1);
    emu.cont();
    assert_eq!(emu.reg_read_word(Reg::R0), 2);
    assert_eq!(emu.reg_read_word(Reg::PC), bin.len() as u16);
}
