
#[cfg(test)]
mod tests {

    use as_lib::assemble;
    use emu_lib::{Emulator, constants::DATA_START};
    use common::asm::Reg;
    
    #[test]
    fn test_literal() {
        let bin = assemble(r#"
            mov #1, r0
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 1);
    }

    #[test]
    fn test_indirect() {
        let bin = assemble(r#"
            mov #0100, r0
            mov @r0, r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o321);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o321);


        let bin = assemble(r#"
            mov #0100, r0
            mov (r0), r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o321);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o321);

        let bin = assemble(r#"
            mov     #0100, r0
            movb    (r0), r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o777);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o177777);
    }

    #[test]
    fn test_autoinc() {
        let bin = assemble(r#"
            mov #0100, r0
            mov (r0)+, r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o321);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o321);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o102);

        let bin = assemble(r#"
            mov     #0100, r0
            movb    (r0)+, r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o121);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o121);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o101);

        let bin = assemble(r#"
            mov     #0100, r0
            movb    (r0)+, r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o777);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o177777);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o101);
    }


    #[test]
    fn test_autodec() {
        let bin = assemble(r#"
            mov #0102, r0
            mov -(r0), r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o321);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o321);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);


        let bin = assemble(r#"
            mov     #0101, r0
            movb    -(r0), r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o777);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o177777);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
    }


    #[test]
    fn test_autoinc_def() {
        let bin = assemble(r#"
            mov #0100, r0
            mov @(r0)+, r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o320);
        emu.mem_write_word(0o320, 0o33);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o33);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o102);

        let bin = assemble(r#"
            mov     #0100, r0
            movb    @(r0)+, r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o320);
        emu.mem_write_word(0o320, 0o33);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o33);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o102);
    }

    #[test]
    fn test_autodec_def() {
        let bin = assemble(r#"
            mov #0102, r0
            mov @-(r0), r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o320);
        emu.mem_write_word(0o320, 0o33);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o33);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);

        let bin = assemble(r#"
            mov     #0102, r0
            movb    @-(r0), r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o320);
        emu.mem_write_word(0o320, 0o33);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o33);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
    }
}
