use as_lib::assemble;
use common::asm::Reg;
use common::constants::DATA_START;
use emu_lib::Emulator;

#[test]
#[should_panic]
fn unaligned_a() {
    let prog = assemble(
        r#"
        jmp start

        . = 11

    start:
        mov #1, r0
        halt
    "#,
    );
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
}

#[test]
#[should_panic]
fn unaligned_b() {
    let prog = assemble(
        r#"
        jmp start

        .byte 0

    start:
        mov #1, r0
        halt
    "#,
    );
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
}

#[test]
fn even() {
    let prog = assemble(
        r#"
        jmp start

        . = 10

        .even
    start:
        mov #1, r0
        halt
    "#,
    );
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 1);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len() as u16
    );

    let prog = assemble(
        r#"
        jmp start

        . = 11

        .even
    start:
        mov #1, r0
        halt
    "#,
    );
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 1);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len() as u16
    );

    let prog = assemble(
        r#"
        jmp start

        .word 0

        .even
    start:
        mov #1, r0
        halt
    "#,
    );
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 1);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len() as u16
    );

    let prog = assemble(
        r#"
        jmp start

        .byte 0

        .even
    start:
        mov #1, r0
        halt
    "#,
    );
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 1);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len() as u16
    );
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
    let prog = assemble(asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, 0);
    emu.run_at(prog.symbols.get("_start").unwrap().val);
    assert_eq!(emu.reg_read_word(Reg::R0), 0);
    emu.cont();
    assert_eq!(emu.reg_read_word(Reg::R0), 1);
    emu.cont();
    assert_eq!(emu.reg_read_word(Reg::R0), 2);
    assert_eq!(emu.reg_read_word(Reg::PC), prog.text.len() as u16);
}
