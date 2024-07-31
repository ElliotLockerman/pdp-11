
#[cfg(test)]
mod tests {

    use as_lib::assemble;
    use emu_lib::{Emulator, constants::DATA_START};
    use common::asm::Reg;
    
    #[test]
    fn test_literal_read() {
        let bin = assemble(r#"
            mov #1, r0
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 1);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }

    #[test]
    fn test_literal_read_byte() {
        let bin = assemble(r#"
            movb #1, r0
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 1);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }

    #[test]
    #[should_panic]
    fn test_literal_write() {
        let bin = assemble(r#"
            mov r0, #1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 1);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }

    #[test]
    fn test_absolute_read() {
        let bin = assemble(r#"
            mov @#020, r0
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o20, 0o321);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o321);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }

    #[test]
    #[should_panic]
    fn test_large_literal() {
        assemble(r#"
            mov #10000000000, r0
        "#);
    }


    #[test]
    fn test_indirect_read() {
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
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


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
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

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
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }

    #[test]
    fn test_indirect_write() {
        let bin = assemble(r#"
            mov #0100, r0
            mov #020, @r0
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o321);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o100), 0o20);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


        let bin = assemble(r#"
            mov #0100, r0
            mov #020, (r0)
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o321);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o100), 0o20);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


        let bin = assemble(r#"
            mov     #0100, r0
            movb    #020, (r0)
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o721);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o100), 0o420);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }
    
    #[test]
    #[should_panic]
    fn test_unaligned() {
        let bin = assemble(r#"
            mov #0101, r0
            mov @r0, r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o321);
        emu.run_at(DATA_START);


        let bin = assemble(r#"
            mov #0101, r0
            mov #020, @r0
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o321);
        emu.run_at(DATA_START);
    }


    #[test]
    fn test_autoinc_read() {
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
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

        let bin = assemble(r#"
            mov     #0100, r0
            movb    (r0)+, r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o7121);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o121);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o101);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

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
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }


    #[test]
    fn test_autoinc_write() {
        let bin = assemble(r#"
            mov #0100, r0
            mov #020, (r0)+
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o321);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o100), 0o20);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o102);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


        let bin = assemble(r#"
            mov     #0100, r0
            movb    #020, (r0)+
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o721);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o100), 0o420);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o101);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }



    #[test]
    fn test_autodec_read() {
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
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


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
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }


    #[test]
    fn test_autodec_write() {
        let bin = assemble(r#"
            mov #0102, r0
            mov #020, -(r0)
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o321);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o100), 0o20);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


        let bin = assemble(r#"
            mov     #0101, r0
            movb    #020, -(r0)
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o721);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o100), 0o420);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }


    #[test]
    fn test_autoinc_def_read() {
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
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

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
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }



    #[test]
    fn test_autoinc_def_write() {
        let bin = assemble(r#"
            mov #0100, r0
            mov #07720, @(r0)+
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o320);
        emu.mem_write_word(0o320, 0o33);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o320), 0o7720);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o102);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


        let bin = assemble(r#"
            mov     #0100, r0
            movb    #020, @(r0)+
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o320);
        emu.mem_write_word(0o320, 0o721);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o320), 0o420);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o102);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }


    #[test]
    fn test_autodec_def_read() {
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
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

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
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }


    #[test]
    fn test_autodec_def_write() {
        let bin = assemble(r#"
            mov #0102, r0
            mov #07720, @-(r0)
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o320);
        emu.mem_write_word(0o320, 0o33);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o320), 0o7720);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


        let bin = assemble(r#"
            mov     #0102, r0
            movb    #020, @-(r0)
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o100, 0o320);
        emu.mem_write_word(0o320, 0o721);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o320), 0o420);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }

    #[test]
    fn test_index_read() {
        let bin = assemble(r#"
            mov #0100, r0
            mov 2(r0), r1
            mov 4(r0), r2
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o102, 0o320);
        emu.mem_write_word(0o104, 0o300);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R2), 0o300);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o320);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


        let bin = assemble(r#"
            mov     #0100, r0
            movb    1(r0), r1
            movb    2(r0), r2
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_byte(0o101, 0o20);
        emu.mem_write_byte(0o102, 0o40);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R2), 0o40);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o20);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }

    #[test]
    fn test_index_write() {
        let bin = assemble(r#"
            mov #0100, r0
            mov #01, 2(r0)
            mov #02, 4(r0)
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o102, 0o320);
        emu.mem_write_word(0o104, 0o300);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o102), 0o1);
        assert_eq!(emu.get_state().mem_read_word(0o104), 0o2);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

        let bin = assemble(r#"
            mov     #0100, r0
            movb    #020, 2(r0)
            movb    #040, 4(r0)
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o102, 0o720);
        emu.mem_write_word(0o104, 0o740);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o102), 0o420);
        assert_eq!(emu.get_state().mem_read_word(0o104), 0o440);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }



    #[test]
    fn test_index_def_read() {
        let bin = assemble(r#"
            mov #0100, r0
            mov @2(r0), r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o102, 0o320);
        emu.mem_write_word(0o320, 0o33);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o33);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


        let bin = assemble(r#"
            mov     #0100, r0
            movb    @2(r0), r1
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o102, 0o320);
        emu.mem_write_word(0o320, 0o720);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R1), 0o177720);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }


    #[test]
    fn test_index_def_write() {
        let bin = assemble(r#"
            mov #0100, r0
            mov #011, @2(r0)
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o102, 0o320);
        emu.mem_write_word(0o320, 0o33);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o320), 0o11);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


        let bin = assemble(r#"
            mov     #0100, r0
            movb    #011, @2(r0)
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.mem_write_word(0o102, 0o320);
        emu.mem_write_word(0o320, 0o740);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().mem_read_word(0o320), 0o411);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o100);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
    }


    #[test]
    fn test_relative_label_read() {
        let bin = assemble(r#"
        label:
            .word 012
            mov label, r0
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START + 2);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o012);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


        let bin = assemble(r#"
        label:
            .word 0533
            movb label, r0
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START + 2);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o133);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

        let bin = assemble(r#"
        label:
            .word 012
            mov label, r0
            halt
        "#);
        let mut emu = Emulator::new();
        let offset = 16;
        emu.load_image(&bin, DATA_START + offset);
        emu.run_at(DATA_START + offset + 2);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o012);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + offset + bin.len() as u16);
    }

    #[test]
    fn test_relative_label_write() {
        let bin = assemble(r#"
        label:
            .word 07777
            mov #012, r0
            mov r0, label
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START + 2);
        assert_eq!(emu.get_state().mem_read_word(DATA_START), 0o012);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

        let bin = assemble(r#"
        label:
            .word 07777
            mov     #012, r0
            movb    r0, label
            halt
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START + 2);
        assert_eq!(emu.get_state().mem_read_word(DATA_START), 0o7412);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


        let bin = assemble(r#"
        label:
            .word 07777
            mov     #012, r0
            movb    r0, label
            halt
        "#);
        let mut emu = Emulator::new();
        let offset = 16;
        emu.load_image(&bin, DATA_START + offset);
        emu.run_at(DATA_START + offset + 2);
        assert_eq!(emu.get_state().mem_read_word(DATA_START + offset), 0o7412);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16 + offset);
    }

    #[test]
    fn test_immediate_label_read() {
        let bin = assemble(r#"
            mov #label, r0
            halt
        label:
            .word 012
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 6);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - 2);
    }

    #[test]
    fn test_relative_def_label_read() {
        let bin = assemble(r#"
        label:
            .word 0410
            mov @label, r0
            halt
            .word 066
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START + 2);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o66);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - 2);

        let bin = assemble(r#"
        label:
            .word 0410
            movb @label, r0
            halt
            .word 0533
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START + 2);
        assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o133);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - 2);
    }


    #[test]
    fn test_relative_def_label_write() {
        let bin = assemble(r#"
        label:
            .word 0410
            mov #033, r0
            mov r0, @label
            halt
            .word 066
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START + 2);
        assert_eq!(emu.get_state().mem_read_word(0o410), 0o33);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - 2);


        let bin = assemble(r#"
        label:
            .word 0414
            mov     #00, r0
            movb    r0, @label
            halt
            .word 07777
        "#);
        let mut emu = Emulator::new();
        emu.load_image(&bin, DATA_START);
        emu.run_at(DATA_START + 2);
        assert_eq!(emu.get_state().mem_read_word(0o414), 0o7400);
        assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - 2);
    }

}
