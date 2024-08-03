
#[cfg(test)]
mod call {

    use as_lib::assemble;
    use emu_lib::Emulator;
    use common::asm::Reg;
    use common::constants::DATA_START;

    // asm must set r0 to 1 if the jsr wasn't successful, 2 if it was
    fn run(asm: &str) {
        let bin = assemble(&asm);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.get_state_mut().reg_write_word(Reg::SP, 0o150000);
        emu.run_at(DATA_START);
        println!("pc: {:o}", emu.get_state().reg_read_word(Reg::PC));
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 2);
    }

    #[test]
    fn test_call_label() {
        run(r#"
            jsr pc, taken

            mov #1, r0
            halt

        taken:
            mov #2, r0
            halt
            
        "#);
    }

    #[test]
    fn test_call_relative() {
        run(r#"
            jsr pc, 12

            mov #1, r0
            halt

            mov #2, r0
            halt
        "#);

        run(r#"
            br start

            mov #1, r0
            halt

        start:
            jsr pc, 14

            mov #2, r0
            halt
        "#);
    }

    #[test]
    fn test_call_abs() {
        run(r#"
            jsr pc, @#412

            mov #1, r0
            halt

            mov #2, r0
            halt
        "#);
    }


    #[test]
    fn test_call_def() {
        run(r#"
            mov #414, r1
            jsr pc, (r1)

            mov #1, r0
            halt

            mov #2, r0
            halt
        "#);
    }


    #[test]
    fn test_call_ret() {
        run(r#"
            mov #1, r0
            jsr pc, fun

            dec r0
            halt

        fun:
            mov #3, r0
            rts pc
            halt
            
        "#);
    }

    #[test]
    fn test_call_link_arg() {
        run(r#"
            mov #1, r0
            jsr r1, fun
            .word 3

            mov r2, r0
            halt

        fun:
            mov (r1)+, r2
            dec r2
            rts r1
        "#)
    }

    #[test]
    fn test_call_stack_arg() {
        run(r#"
            mov #1, r0
            mov #3, -(sp)
            jsr pc, fun

            mov (sp)+, r0
            halt

        fun:
            dec 2(sp)
            rts pc
        "#);
    }

}

