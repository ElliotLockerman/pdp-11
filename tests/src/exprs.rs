use as_lib::assemble;
use emu_lib::Emulator;
use common::asm::Reg;
use common::constants::DATA_START;

fn _eval(expr: &str, r0_exp: u16, word: bool) {
    let ch = if word { ' ' } else { 'b' };
    let asm = format!(r#"
        SYM = {expr}
        mov{ch} #SYM, r0
        halt
    "#);
    let bin = assemble(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.get_state().reg_read_word(Reg::R0), r0_exp);
    assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

fn eval_word(expr: &str, r0_exp: u16) {
    _eval(expr, r0_exp, true);
}

fn eval_byte(expr: &str, r0_exp: u16) {
    _eval(expr, r0_exp, false);
}


#[test]
fn literal() {
    eval_word(r"1", 0o1);
    eval_word(r"177777", 0o177777);

    eval_byte(r"1", 0o1);
    eval_byte(r"177", 0o177);
    eval_byte(r"377", 0o177777);
}

#[test]
#[should_panic]
fn literal_large() {
    eval_word(r"277777", 0o0);
}

#[test]
#[should_panic]
fn literal_large_byte() {
    eval_byte(r"477", 0o477);
}

#[test]
fn add() {
    eval_word(r"2 + 3", 0o5);
    eval_word(r"177777 + 1", 0);

    eval_byte(r"2 + 3", 0o5);
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

    eval_word(r"17003 & 6", 0o2);
    eval_word(r"17004 & 2", 0o0);

    eval_byte(r"17003 & 17006", 0o2);
    eval_byte(r"17004 & 17002", 0o0);
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
fn array_len() {
    let asm = r#"
        br start

    arr:
        .word 1, 2, 3, 4, 5, 6
        len = . - arr

    start:
        mov #len, r0
        halt
    "#;
    let bin = assemble(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o14);
    assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


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
    let bin = assemble(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o14);
    assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);



}
