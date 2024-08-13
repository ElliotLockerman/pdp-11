use as_lib::assemble;
use emu_lib::Emulator;
use common::asm::Reg;
use common::constants::DATA_START;
use crate::flags::{Flags, flags};


#[test]
fn mul_full() {
    fn run(
        r0_init: u16,
        r2_init: u16,
        result_exp: u32,
        flags_exp: Flags,
    ) {
        let asm = format!(r#"
            mul r0, r2
            halt
        "#);
        let bin = assemble(&asm);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.get_state_mut().reg_write_word(Reg::R0, r0_init);
        emu.get_state_mut().reg_write_word(Reg::R2, r2_init);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), result_exp as u16);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), (result_exp >> u16::BITS) as u16);
        assert_eq!(emu.get_state().reg_read_word(Reg::R2), r2_init);
        let status = emu.get_state().get_status();
        assert_eq!(status.get_carry(), flags_exp.c, "carry flag");
        assert_eq!(status.get_overflow(),flags_exp.v, "overflow flag");
        assert_eq!(status.get_zero(), flags_exp.z, "zero flag");
        assert_eq!(status.get_negative(), flags_exp.n, "negative flag");
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }


    run(0, 0, 0, flags().z());
    run(1, 1, 1, flags());
    run(0, 1, 0, flags().z());
    run(3, 5, 15, flags());
    run(-1i16 as u16, 0o1, -1i32 as u32, flags().n());
    run(0o7, -1i16 as u16, -7i32 as u32, flags().n());
    run(0o077777, 0o2, 0o177776, flags().c());
    run(i16::MIN as u16, 0o2, ((i16::MIN as i32) * 2) as u32, flags().c().n());
}

#[test]
fn mul_lower() {
    fn run(
        r3_init: u16,
        r4_init: u16,
        r3_exp: u16,
        flags_exp: Flags,
    ) {
        let asm = r#"
            mul r3, r4
            halt
        "#;
        let bin = assemble(&asm);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.get_state_mut().reg_write_word(Reg::R3, r3_init);
        emu.get_state_mut().reg_write_word(Reg::R4, r4_init);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R3), r3_exp);
        assert_eq!(emu.get_state().reg_read_word(Reg::R4), r4_init);
        let status = emu.get_state().get_status();
        assert_eq!(status.get_carry(), flags_exp.c, "carry flag");
        assert_eq!(status.get_overflow(),flags_exp.v, "overflow flag");
        assert_eq!(status.get_zero(), flags_exp.z, "zero flag");
        assert_eq!(status.get_negative(), flags_exp.n, "negative flag");
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }


    run(0, 0, 0, flags().z());
    run(1, 1, 1, flags());
    run(0, 1, 0, flags().z());
    run(3, 5, 15, flags());
    run(-1i16 as u16, 0o1, -1i16 as u16, flags().n());
    run(0o7, -1i16 as u16, -7i32 as u16, flags().n());
    run(0o077777, 0o2, 0o177776, flags().c());
    run(i16::MIN as u16, 0o2, ((i16::MIN as i32) * 2) as u16, flags().c().n());
}

#[test]
fn xor() {
    fn run(
        r0_init: u16,
        r1_init: u16,
        r1_exp: u16,
        flags_exp: Flags,
    ) {
        let asm = r#"
            xor r0, r1
            halt
        "#;
        let bin = assemble(&asm);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.get_state_mut().reg_write_word(Reg::R0, r0_init);
        emu.get_state_mut().reg_write_word(Reg::R1, r1_init);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), r0_init);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), r1_exp);
        let status = emu.get_state().get_status();
        assert_eq!(status.get_carry(), flags_exp.c, "carry flag");
        assert_eq!(status.get_overflow(),flags_exp.v, "overflow flag");
        assert_eq!(status.get_zero(), flags_exp.z, "zero flag");
        assert_eq!(status.get_negative(), flags_exp.n, "negative flag");
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }

    run(0, 0, 0, flags().z());
    run(1, 1, 0, flags().z());
    run(0, 1, 1, flags());
    run(0o5, 0o2, 0o7, flags());
    run(0o5, 0o1, 0o4, flags());
    run(0o177777, 0o2, 0o177775, flags().n());
    run(0o177777, 0o100000, 0o077777, flags());
}
