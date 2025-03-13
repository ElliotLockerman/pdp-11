use crate::flags::{C, N, V, Z, check_flags};
use as_lib::assemble_raw;
use common::asm::Reg;
use common::constants::DATA_START;
use common::misc::ToU16P;
use emu_lib::Emulator;

#[test]
fn mul_full() {
    fn run(r0_init: u16, r2_init: u16, result_exp: u32, flags_exp: u16) {
        let asm = format!(
            r#"
            mul r2, r0
            halt
        "#
        );
        let prog = assemble_raw(&asm);
        let mut emu = Emulator::new();
        emu.load_image(&prog.text, DATA_START);
        emu.reg_write_word(Reg::R0, r0_init);
        emu.reg_write_word(Reg::R2, r2_init);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), result_exp as u16);
        assert_eq!(
            emu.reg_read_word(Reg::R1),
            (result_exp >> u16::BITS).to_u16p()
        );
        assert_eq!(emu.reg_read_word(Reg::R2), r2_init);
        check_flags(&emu, flags_exp);
        assert_eq!(
            emu.reg_read_word(Reg::PC),
            DATA_START + prog.text.len().to_u16p()
        );
    }

    run(0, 0, 0, Z);
    run(1, 1, 1, 0);
    run(0, 1, 0, Z);
    run(3, 5, 15, 0);
    run(-1i16 as u16, 0o1, -1i32 as u32, N);
    run(0o7, -1i16 as u16, -7i32 as u32, N);
    run(0o077777, 0o2, 0o177776, C);
    run(i16::MIN as u16, 0o2, ((i16::MIN as i32) * 2) as u32, C | N);
    run(
        i16::MIN as u16,
        -1i16 as u16,
        (-(i16::MIN as i32)) as u32,
        C,
    );
    run(
        i16::MIN as u16,
        -2i16 as u16,
        (-2 * (i16::MIN as i32)) as u32,
        C,
    );
}

#[test]
fn mul_lower() {
    fn run(r3_init: u16, r4_init: u16, r3_exp: u16, flags_exp: u16) {
        let asm = r#"
            mul r4, r3
            halt
        "#;
        let prog = assemble_raw(&asm);
        let mut emu = Emulator::new();
        emu.load_image(&prog.text, DATA_START);
        emu.reg_write_word(Reg::R3, r3_init);
        emu.reg_write_word(Reg::R4, r4_init);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R3), r3_exp);
        assert_eq!(emu.reg_read_word(Reg::R4), r4_init);
        check_flags(&emu, flags_exp);
        assert_eq!(
            emu.reg_read_word(Reg::PC),
            DATA_START + prog.text.len().to_u16p()
        );
    }

    run(0, 0, 0, Z);
    run(1, 1, 1, 0);
    run(0, 1, 0, Z);
    run(3, 5, 15, 0);
    run(-1i16 as u16, 0o1, -1i16 as u16, N);
    run(0o7, -1i16 as u16, -7i32 as u16, N);
    run(0o077777, 0o2, 0o177776, C);
    run(i16::MIN as u16, 0o2, ((i16::MIN as i32) * 2) as u16, C | N);
    run(
        i16::MIN as u16,
        -1i16 as u16,
        (-(i16::MIN as i32)) as u16,
        C,
    );
    run(i16::MIN as u16, -2i16 as u16, 0, C);
}

#[test]
#[should_panic]
fn div_odd() {
    assemble_raw("div r2, r1");
}

#[test]
fn div() {
    fn run(dividend: u32, divisor: u16, quot_exp: u16, rem_exp: u16, flags_exp: u16) {
        let asm = format!(
            r#"
            div r2, r0
            halt
        "#
        );
        let prog = assemble_raw(&asm);
        let mut emu = Emulator::new();
        emu.load_image(&prog.text, DATA_START);
        emu.reg_write_word(Reg::R0, dividend as u16);
        emu.reg_write_word(Reg::R1, (dividend >> u16::BITS).to_u16p());
        emu.reg_write_word(Reg::R2, divisor);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), quot_exp, "quot");
        assert_eq!(emu.reg_read_word(Reg::R1), rem_exp, "rem");
        assert_eq!(emu.reg_read_word(Reg::R2), divisor);
        check_flags(&emu, flags_exp);
        assert_eq!(
            emu.reg_read_word(Reg::PC),
            DATA_START + prog.text.len().to_u16p()
        );
    }

    run(0, 1, 0, 0, Z);
    run(1, 1, 1, 0, 0);
    run(1, 0, 1, 0, V | C);
    run(2, 1, 2, 0, 0);
    run(3, 2, 1, 1, 0);
    run(1, 2, 0, 1, Z);
    run(-2i32 as u32, 1, -2i16 as u16, 0, N);
    run(-3i32 as u32, 2, -1i16 as u16, -1i16 as u16, N);
    run(
        i32::MIN as u32,
        1,
        i32::MIN as u16,
        ((i32::MIN as u32) >> u16::BITS).to_u16p(),
        N | V,
    );
}

#[test]
fn ash() {
    fn run(val: i16, shift: i16, val_exp: i16, flags_exp: u16) {
        let asm = format!(
            r#"
            ash r0, r1
            halt
        "#
        );
        let prog = assemble_raw(&asm);
        let mut emu = Emulator::new();
        emu.load_image(&prog.text, DATA_START);
        emu.reg_write_word(Reg::R0, shift as u16);
        emu.reg_write_word(Reg::R1, val as u16);
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), shift as u16, "shift (after)");
        assert_eq!(emu.reg_read_word(Reg::R1), val_exp as u16, "val (after)");
        check_flags(&emu, flags_exp);
        assert_eq!(
            emu.reg_read_word(Reg::PC),
            DATA_START + prog.text.len().to_u16p()
        );
    }

    run(0o0, 0o0, 0o0, Z);
    run(-1i16, 0o0, -1i16, N);
    run(0o0, 0o1, 0o0, Z);
    run(0o1, 0o1, 0o2, 0);
    run(0o001234, 0o3, 0o012340, 0);
    run(0o100000u16 as i16, 0o1, 0o0u16 as i16, V | C | Z);
    run(0o100001u16 as i16, 0o1, 0o2u16 as i16, V | C);
    run(0o040000, 0o1, 0o100000u16 as i16, V | N);
    run(0o140000u16 as i16, 0o1, 0o100000u16 as i16, C | N);

    run(0o0, -0o1, 0o0, Z);
    run(0o1, -0o1, 0o0, Z | C);
    run(0o2, -0o1, 0o1, 0);
    run(0o3, -0o1, 0o1, C);
    run(0o100000u16 as i16, -0o1, 0o140000u16 as i16, N);
}

#[test]
fn ashc() {
    fn run(val: i32, shift: i16, val_exp: i32, flags_exp: u16) {
        let asm = format!(
            r#"
            ashc r0, r2
            halt
        "#
        );
        let prog = assemble_raw(&asm);
        let mut emu = Emulator::new();
        emu.load_image(&prog.text, DATA_START);
        emu.reg_write_word(Reg::R0, shift as u16);
        emu.reg_write_word(Reg::R2, val as u16);
        emu.reg_write_word(Reg::R3, ((val as u32) >> u16::BITS).to_u16p());
        emu.run_at(DATA_START);
        assert_eq!(emu.reg_read_word(Reg::R0), shift as u16, "shift (after)");
        let out_lower = emu.reg_read_word(Reg::R2) as u32;
        let out_upper = emu.reg_read_word(Reg::R3) as u32;
        assert_eq!(
            (out_upper << u16::BITS) | out_lower,
            val_exp as u32,
            "val (after)"
        );
        check_flags(&emu, flags_exp);
        assert_eq!(
            emu.reg_read_word(Reg::PC),
            DATA_START + prog.text.len().to_u16p()
        );
    }

    run(0o0, 0o0, 0o0, Z);
    run(-1, 0o0, -1, N);
    run(0o0, 0o1, 0o0, Z);
    run(0o1, 0o1, 0o2, 0);
    run(0o001234, 0o3, 0o012340, 0);
    run(0o040000, 0o1, 0o100000, 0);
    run(0o140000, 0o1, 0o300000, 0);
    run(0o140000, 0o5, 0o140000 << 5, 0);
    run(i32::MIN, 0o1, 0o0, V | C | Z);
    run(i32::MIN + 1, 0o1, 0o2, V | C);
    run(i32::MAX, 0o1, 0i32 - 2, V | N);

    run(0o0, -0o1, 0o0, Z);
    run(0o1, -0o1, 0o0, Z | C);
    run(0o2, -0o1, 0o1, 0);
    run(0o3, -0o1, 0o1, C);
    run(0o100000, -0o1, 0o040000, 0);
    run(0o140000, -0o5, 0o140000 >> 5, 0);
    run(-1i32, -0o1, -1i32, C | N);
    run(i32::MIN, -0o1, i32::MIN >> 1, N);
}

#[test]
fn xor() {
    fn run(r0_init: u16, r1_init: u16, r1_exp: u16, flags_exp: u16) {
        let asm = r#"
            xor r1, r0
            halt
        "#;
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
            DATA_START + prog.text.len().to_u16p()
        );
    }

    run(0, 0, 0, Z);
    run(1, 1, 0, Z);
    run(0, 1, 1, 0);
    run(0o5, 0o2, 0o7, 0);
    run(0o5, 0o1, 0o4, 0);
    run(0o177777, 0o2, 0o177775, N);
    run(0o177777, 0o100000, 0o077777, 0);
}
