use as_lib::assemble_raw;
use common::asm::Reg;
use common::constants::DATA_START;
use common::mem::ToU16P;
use emu_lib::Emulator;

#[test]
fn literal_to_abs() {
    let prog = assemble_raw(
        r#"
        mov #0753, @#020
        halt
    "#,
    );
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o20), 0o753);
    assert_eq!(
        emu.reg_read_word(Reg::PC),
        DATA_START + prog.text.len().to_u16p()
    );
}

#[test]
fn double_autoinc() {
    let prog = assemble_raw(
        r#"
        mov #arr_a, r0
        mov #arr_b, r1

        mov (r0)+, (r1)+
        mov (r0)+, (r1)+

        mov #arr_b, r1
        mov (r1)+, r2
        mov (r1)+, r3
        halt

    arr_a:
        .word 01, 02
    arr_b:
        .word 07, 07
    "#,
    );
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, 0);
    emu.run_at(0);
    assert_eq!(emu.reg_read_word(Reg::R2), 0o1);
    assert_eq!(emu.reg_read_word(Reg::R3), 0o2);
}

#[test]
fn index_autoinc() {
    let prog = assemble_raw(
        r#"
        mov #arr_a, r0
        mov #arr_b, r1

        mov 4(r0), (r1)+
        mov 6(r0), (r1)+

        mov #arr_b, r1
        mov (r1)+, r2
        mov (r1)+, r3
        halt

    arr_a:
        .word 00, 00, 01, 02
    arr_b:
        .word 07, 07
    "#,
    );
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, 0);
    emu.run_at(0);
    assert_eq!(emu.reg_read_word(Reg::R2), 0o1);
    assert_eq!(emu.reg_read_word(Reg::R3), 0o2);
}
