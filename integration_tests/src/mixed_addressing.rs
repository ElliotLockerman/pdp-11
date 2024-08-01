

#[cfg(test)]
mod tests {

    use as_lib::assemble;
    use emu_lib::Emulator;
    use common::asm::Reg;
    use common::constants::DATA_START;

    #[test]
    fn test_literal_to_abs() {
        let bin = assemble(r#"
            mov #0753, @#020
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o20), 0o753);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }

    #[test]
    fn test_double_autoinc() {
        let bin = assemble(r#"
            mov #arr_a, r0
            mov #arr_b, r1

            mov (r0)+, (r1)+
            mov (r0)+, (r1)+

            mov #arr_b, r1
            mov (r1)+, r2
            mov (r1)+, r3
            halt

        arr_a:
            .word 01 02
        arr_b:
            .word 07 07
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, 0);
        emu.run_at(0);
        assert_eq!(emu.get_state().reg_read_word(Reg::R2), 0o1);
        assert_eq!(emu.get_state().reg_read_word(Reg::R3), 0o2);
    }

    #[test]
    fn test_index_autoinc() {
        let bin = assemble(r#"
            mov #arr_a, r0
            mov #arr_b, r1

            mov 4(r0), (r1)+
            mov 6(r0), (r1)+

            mov #arr_b, r1
            mov (r1)+, r2
            mov (r1)+, r3
            halt

        arr_a:
            .word 00 00 01 02
        arr_b:
            .word 07 07
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, 0);
        emu.run_at(0);
        assert_eq!(emu.get_state().reg_read_word(Reg::R2), 0o1);
        assert_eq!(emu.get_state().reg_read_word(Reg::R3), 0o2);
    }
}
