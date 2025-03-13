use as_lib::assemble_raw;
use common::asm::Reg;
use common::constants::DATA_START;
use common::misc::ToU16P;
use emu_lib::Emulator;

fn eval_word(expr: &str, r0_exp: u16) {
    let asm = format!(
        r#"
        SYM = {expr}
        mov #SYM, r0
        halt
    "#
    );
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), r0_exp);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p()
    );
}

fn eval_byte(expr: &str, exp: u8) {
    // Go through memory to avoid sign-extension for movb to reg
    let asm = format!(
        r#"
        SYM = {expr}
        movb    #SYM, data
        movb    data, r0
        mov     data, r1
        halt
    data:
        .byte 0, 0
    "#
    );
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), exp as i8 as i16 as u16);
    assert_eq!(emu.reg_read_word(Reg::R1), exp as u16);
    assert_eq!(
        emu.mem_read_byte(DATA_START + prog.text.len().to_u16p() - 2),
        exp
    );
    assert_eq!(
        emu.mem_read_word(DATA_START + prog.text.len().to_u16p() - 2),
        exp as u16,
    );
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p() - 2
    );
}

#[test]
fn literal() {
    eval_word(r"1", 0o1);
    eval_word(r"177777", 0o177777);
    eval_word(r"-1", 0o177777);
    eval_word(r"-2", 0o177776);

    eval_byte(r"1", 0o1);
    eval_byte(r"177", 0o177);
    eval_byte(r"377", 0o377);

    let asm = format!(
        r#"
        SYM = -1
        ; Go through memory to avoid sign-extension for movb to reg
        movb    #SYM, data
        movb    data, r0
        mov     data, r1
        halt
    data:
        .word 0
    "#
    );
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o177777);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o377);
    assert_eq!(
        emu.mem_read_word(DATA_START + prog.text.len().to_u16p() - 2),
        0o377
    );
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p() - 2
    );
}

#[test]
#[should_panic]
fn literal_large() {
    eval_word(r"277777", 0o0);
}

#[test]
#[should_panic]
fn literal_large_byte() {
    eval_byte(r"477", 0o0);
}

#[test]
fn add() {
    eval_word(r"2 + 3", 0o5);
    eval_word(r"-2 + 3", 0o1);
    eval_word(r"-5 + 2", 0o177775);
    eval_word(r"177777 + 1", 0);

    eval_byte(r"2 + 3", 0o5);
    eval_byte(r"-5 + 3", 0o376);
    eval_byte(r"377 + 3", 0o2);
}

#[test]
fn sub() {
    eval_word(r"4 - 2", 0o2);
    eval_word(r"2 - 3", 0o177777);
}

#[test]
fn and() {
    eval_word(r"3 & 6", 0o2);
    eval_word(r"4 & 2", 0o0);
    eval_word(r"4 & 0", 0o0);
    eval_word(r"4 & -1", 0o4);

    eval_word(r"17003 & 6", 0o2);
    eval_word(r"17004 & 2", 0o0);

    eval_byte(r"17003 & 17006", 0o2);
    eval_byte(r"17004 & 17002", 0o0);
    eval_byte(r"270 & 207", 0o200);
}

#[test]
fn or() {
    eval_word(r"1 ! 1", 0o1);
    eval_word(r"1 ! 2", 0o3);

    eval_byte(r"7001 ! 7001", 0o1);
    eval_byte(r"7001 ! 7002", 0o3);
}

#[test]
fn compound() {
    eval_word(r"1 + 1 - 1", 0o1);
    eval_word(r"1 + 2 - 1", 0o2);
    eval_word(r"1 + 1 ! 2", 0o2);
    eval_word(r"1 ! 2 + 1", 0o4);
    eval_word(r"1 + 1 & 2", 0o2);
    eval_word(r"1 & 2 + 1", 0o1);
}

#[test]
fn malformed() {
    use std::panic::catch_unwind;
    fn fail(asm: &str) {
        if let Ok(_) = catch_unwind(|| assemble_raw(asm)) {
            panic!("Failure: was supposed to panic");
        }
    }

    // Most of these are allowed in PAL-11, with missing operators being replaced with + and
    // missing operands being replaced 0. I've chosen to reject them.
    fail(r"SYM = +");
    fail(r"SYM = 1 +");
    fail(r"SYM = + 1");
    fail(r"SYM = 1 1");
    fail(r"SYM = + !");
    fail(r"mov 1 1(r0), r1");
    fail(r"mov !(r0), r1");
    fail(r"mov 1!(r0), r1");
}

#[test]
fn array_len() {
    let asm = r#"
        . = 400
    arr:
        .word 1, 2, 3, 4, 5, 6
        len = . - arr

    _start:
        mov #len, r0
        halt
    "#;
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, 0);
    emu.run_at(prog.symbols.get("_start").unwrap().val);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o14);
    assert_eq!(emu.reg_read_word(Reg::PC), prog.text.len().to_u16p());

    let asm = r#"
        br start

    arr:
        .word 1, 2, 3, 4, 5, 6
    len: 
        .word . - arr

    start:
        mov len, r0
        halt
    "#;
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o14);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p()
    );
}

#[test]
fn relocation() {
    let asm = r#"
        br start

    val:
        .word 34

    start:
        mov #66, val 
        halt
    "#;
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(DATA_START + 0o2), 0o66);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p()
    );

    let asm = r#"
        br start

    val:
        .word 34

    start:
        mov #66, val 
        halt
    "#;
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, 0);
    emu.run();
    assert_eq!(emu.mem_read_word(0o2), 0o66);
    assert_eq!(emu.reg_read_word(Reg::PC), prog.text.len().to_u16p());

    let asm = r#"
        . = 400

        br start

    val:
        .word 34

    start:
        mov #66, val 
        halt
    "#;
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, 0);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(DATA_START + 0o2), 0o66);
    assert_eq!(emu.reg_read_word(Reg::PC), prog.text.len().to_u16p());
}

#[test]
fn period_unchanged() {
    let asm = r#"
        . = 400

        br start

    val:
        .word 34

        . = .

    start:
        mov #66, val 
        halt
    "#;
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, 0);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(DATA_START + 0o2), 0o66);
    assert_eq!(emu.reg_read_word(Reg::PC), prog.text.len().to_u16p());
}

// Setting the location to a lower value may very well be allowed, but I've chosen to not support
// it.
#[test]
#[should_panic]
fn decreasing() {
    let asm = r#"
        . = 1
        . = 0
    "#;
    assemble_raw(&asm);
}

#[test]
fn reloc_label_reads() {
    let asm = r#"
        . = 400
        loc = 100
        mov loc, r0
        mov #loc, r1
        halt
    "#;
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, 0);
    emu.mem_write_word(0o100, 0o123);
    emu.mem_write_word(DATA_START + 0o100, 0o333);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o123);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), prog.text.len().to_u16p());

    let asm = r#"
        loc = 100
        mov loc, r0
        mov #loc, r1
        halt
    "#;
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.mem_write_word(0o100, 0o123);
    emu.mem_write_word(DATA_START + 0o100, 0o333);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o333);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o100);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p()
    );

    let asm = r#"
        loc = 100
        mov @#loc, r0
        mov #loc, r1
        halt
    "#;
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.mem_write_word(0o100, 0o123);
    emu.mem_write_word(DATA_START + 0o100, 0o333);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o123);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o100);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p()
    );
}

#[test]
fn update_symbol() {
    let asm = r#"
        sym = 100
        mov #sym, r0
        sym = 100 + 1
        mov #sym, r1
        halt
    "#;
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o101);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p()
    );
}

#[test]
fn forward() {
    let asm = r#"
        br start

        elem = arr + 4
        arr:
            .word 0, 0, 27, 0

    start:
        mov elem, r0
        halt
    "#;
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o27);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p()
    );
}

#[test]
#[should_panic]
fn double_forward() {
    let asm = r#"
        mov elem, r0
        halt

        elem = arr + 4
        arr:
            .word 0, 0, 27, 0

    "#;
    let prog = assemble_raw(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o27);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p()
    );
}
