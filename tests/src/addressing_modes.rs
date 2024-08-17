
use as_lib::{assemble, assemble_with_symbols};
use emu_lib::Emulator;
use common::asm::Reg;
use common::constants::{DATA_START, WORD_SIZE};

#[test]
fn literal_read() {
    let bin = assemble(r#"
        mov #1, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o1);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        mov #10., r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o12);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn regs() {
    let bin = assemble(r#"
        clr r0

        mov r0, r1
        inc r1

        mov r1, r2
        inc r2

        mov r2, r3
        inc r3

        mov r3, r4
        inc r4

        mov r4, r5
        inc r5

        mov #1000, r6

        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o0);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o1);
    assert_eq!(emu.reg_read_word(Reg::R2), 0o2);
    assert_eq!(emu.reg_read_word(Reg::R3), 0o3);
    assert_eq!(emu.reg_read_word(Reg::R4), 0o4);
    assert_eq!(emu.reg_read_word(Reg::R5), 0o5);
    assert_eq!(emu.reg_read_word(Reg::SP), 0o1000);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn neg_literal_read() {
    let bin = assemble(r#"
        mov #-1, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), -0o1i16 as u16);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        mov #-10., r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), -0o12i16 as u16);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn literal_read_byte() {
    let bin = assemble(r#"
        movb #1, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 1);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn char_literal_read() {
    let bin = assemble(r#"
        mov #177777, r0
        mov #'a, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0x61);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn char_literal_read_byte() {
    let bin = assemble(r#"
        mov     #177777, r0
        movb    #'a, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0x61);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
#[should_panic]
fn literal_write() {
    let bin = assemble(r#"
        mov r0, #1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 1);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn absolute_read() {
    let bin = assemble(r#"
        mov @#20, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o20, 0o321);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o321);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
#[should_panic]
fn large_literal() {
    assemble(r#"
        mov #10000000000, r0
    "#);
}


#[test]
fn indirect_read() {
    let bin = assemble(r#"
        mov #100, r0
        mov @r0, r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o321);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o321);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
        mov #100, r0
        mov (r0), r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o321);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o321);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        mov     #100, r0
        movb    (r0), r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o777);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o177777);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        loc = 100
        mov     #loc, r0
        movb    (r0), r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o777);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o177777);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn indirect_write() {
    let bin = assemble(r#"
        mov #100, r0
        mov #20, @r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o321);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o100), 0o20);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
        mov #100, r0
        mov #20, (r0)
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o321);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o100), 0o20);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
        mov     #100, r0
        movb    #20, (r0)
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o721);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o100), 0o420);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
#[should_panic]
fn unaligned() {
    let bin = assemble(r#"
        mov #101, r0
        mov @r0, r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o321);
    emu.run_at(DATA_START);


    let bin = assemble(r#"
        mov #101, r0
        mov #20, @r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o321);
    emu.run_at(DATA_START);
}


#[test]
fn autoinc_read() {
    let bin = assemble(r#"
        mov #100, r0
        mov (r0)+, r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o321);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o321);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o102);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        mov     #100, r0
        movb    (r0)+, r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o7121);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o121);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o101);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        mov     #100, r0
        movb    (r0)+, r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o777);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o177777);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o101);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}


#[test]
fn autoinc_write() {
    let bin = assemble(r#"
        mov #100, r0
        mov #20, (r0)+
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o321);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o100), 0o20);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o102);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
        mov     #100, r0
        movb    #20, (r0)+
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o721);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o100), 0o420);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o101);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}



#[test]
fn autodec_read() {
    let bin = assemble(r#"
        mov #102, r0
        mov -(r0), r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o321);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o321);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
        mov     #101, r0
        movb    -(r0), r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o777);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o177777);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}


#[test]
fn autodec_write() {
    let bin = assemble(r#"
        mov #102, r0
        mov #20, -(r0)
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o321);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o100), 0o20);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
        mov     #101, r0
        movb    #20, -(r0)
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o721);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o100), 0o420);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}


#[test]
fn autoinc_def_read() {
    let bin = assemble(r#"
        mov #100, r0
        mov @(r0)+, r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o320);
    emu.mem_write_word(0o320, 0o33);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o33);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o102);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        mov     #100, r0
        movb    @(r0)+, r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o320);
    emu.mem_write_word(0o320, 0o33);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o33);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o102);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}



#[test]
fn autoinc_def_write() {
    let bin = assemble(r#"
        mov #100, r0
        mov #7720, @(r0)+
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o320);
    emu.mem_write_word(0o320, 0o33);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o320), 0o7720);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o102);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
        mov     #100, r0
        movb    #20, @(r0)+
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o320);
    emu.mem_write_word(0o320, 0o721);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o320), 0o420);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o102);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}


#[test]
fn autodec_def_read() {
    let bin = assemble(r#"
        mov #102, r0
        mov @-(r0), r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o320);
    emu.mem_write_word(0o320, 0o33);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o33);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        mov     #102, r0
        movb    @-(r0), r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o320);
    emu.mem_write_word(0o320, 0o33);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o33);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}


#[test]
fn autodec_def_write() {
    let bin = assemble(r#"
        mov #102, r0
        mov #7720, @-(r0)
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o320);
    emu.mem_write_word(0o320, 0o33);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o320), 0o7720);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
        mov     #102, r0
        movb    #20, @-(r0)
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o100, 0o320);
    emu.mem_write_word(0o320, 0o721);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o320), 0o420);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn index_read() {
    let bin = assemble(r#"
        mov #100, r0
        mov 2(r0), r1
        mov 4(r0), r2
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o102, 0o320);
    emu.mem_write_word(0o104, 0o300);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R2), 0o300);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o320);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        FIELD_A = 2
        FIELD_B = 4
        mov #100, r0
        mov FIELD_A(r0), r1
        mov FIELD_B(r0), r2
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o102, 0o320);
    emu.mem_write_word(0o104, 0o300);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R2), 0o300);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o320);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        FIELD_A = 2
        mov #100, r0
        mov FIELD_A(r0), r1
        mov FIELD_A + 2(r0), r2
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o102, 0o320);
    emu.mem_write_word(0o104, 0o300);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R2), 0o300);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o320);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        mov     #100, r0
        movb    1(r0), r1
        movb    2(r0), r2
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_byte(0o101, 0o20);
    emu.mem_write_byte(0o102, 0o40);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R2), 0o40);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o20);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
    fn neg_index_read() {
    let bin = assemble(r#"
        mov #106, r0
        mov -4(r0), r1
        mov -2(r0), r2
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o102, 0o320);
    emu.mem_write_word(0o104, 0o300);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R2), 0o300);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o320);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o106);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        FIELD_A = -2
        FIELD_B = -4
        mov #106, r0
        mov FIELD_B(r0), r1
        mov FIELD_A(r0), r2
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o102, 0o320);
    emu.mem_write_word(0o104, 0o300);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R2), 0o300);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o320);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o106);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
        mov     #103, r0
        movb    -2(r0), r1
        movb    -1(r0), r2
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_byte(0o101, 0o20);
    emu.mem_write_byte(0o102, 0o40);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R2), 0o40);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o20);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o103);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn index_write() {
    let bin = assemble(r#"
        mov #100, r0
        mov #1, 2(r0)
        mov #2, 4(r0)
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o102, 0o320);
    emu.mem_write_word(0o104, 0o300);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o102), 0o1);
    assert_eq!(emu.mem_read_word(0o104), 0o2);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
        mov     #100, r0
        movb    #20, 2(r0)
        movb    #40, 4(r0)
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o102, 0o720);
    emu.mem_write_word(0o104, 0o740);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o102), 0o420);
    assert_eq!(emu.mem_read_word(0o104), 0o440);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}



#[test]
fn index_def_read() {
    let bin = assemble(r#"
        mov #100, r0
        mov @2(r0), r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o102, 0o320);
    emu.mem_write_word(0o320, 0o33);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o33);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
        mov     #100, r0
        movb    @2(r0), r1
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o102, 0o320);
    emu.mem_write_word(0o320, 0o720);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o177720);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}


#[test]
fn index_def_write() {
    let bin = assemble(r#"
        mov #100, r0
        mov #11, @2(r0)
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o102, 0o320);
    emu.mem_write_word(0o320, 0o33);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o320), 0o11);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
        mov     #100, r0
        movb    #11, @2(r0)
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.mem_write_word(0o102, 0o320);
    emu.mem_write_word(0o320, 0o740);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(0o320), 0o411);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o100);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}


#[test]
fn relative_label_read() {
    let bin = assemble(r#"
    label:
        .word 012
        mov label, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START + 2);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o012);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
    label:
        .word 0533
        movb label, r0
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START + 2);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o133);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

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
    assert_eq!(emu.reg_read_word(Reg::R0), 0o012);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + offset + bin.len() as u16);
}

#[test]
fn relative_label_write() {
    let bin = assemble(r#"
    label:
        .word 07777
        mov #12, r0
        mov r0, label
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START + WORD_SIZE);
    assert_eq!(emu.mem_read_word(DATA_START), 0o012);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);

    let bin = assemble(r#"
    label:
        .word 07777
        mov     #12, r0
        movb    r0, label
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START + WORD_SIZE);
    assert_eq!(emu.mem_read_word(DATA_START), 0o7412);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);


    let bin = assemble(r#"
    label:
        .word 07777
        mov     #12, r0
        movb    r0, label
        halt
    "#);
    let mut emu = Emulator::new();
    let offset = 16;
    emu.load_image(&bin, DATA_START + offset);
    emu.run_at(DATA_START + offset + 2);
    assert_eq!(emu.mem_read_word(DATA_START + offset), 0o7412);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 + offset);
}

#[test]
fn immediate_label_read() {
    let bin = assemble(r#"
        mov #label, r0
        halt
    label:
        .word 012
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 6);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);
}

#[test]
fn relative_def_label_read() {
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
    assert_eq!(emu.reg_read_word(Reg::R0), 0o66);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);

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
    assert_eq!(emu.reg_read_word(Reg::R0), 0o133);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);
}


#[test]
fn relative_def_label_write() {
    let bin = assemble(r#"
    label:
        .word 0410
        mov #33, r0
        mov r0, @label
        halt
        .word 066
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START + 2);
    assert_eq!(emu.mem_read_word(0o410), 0o33);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);


    let bin = assemble(r#"
    label:
        .word 0414
        mov     #0, r0
        movb    r0, @label
        halt
        .word 07777
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START + 2);
    assert_eq!(emu.mem_read_word(0o414), 0o7400);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);
}

#[test]
fn relative_read() {
    let bin = assemble(r#"
        mov 06, r0
        halt
        .word 066
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o66);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);

    let bin = assemble(r#"
        movb 06, r0
        halt
        .word 0533
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o133);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);
}

#[test]
fn relative_write() {
    let bin = assemble(r#"
        mov #11, r0
        mov r0, 012
        halt
        .word 033
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.mem_read_word(DATA_START + 0o12), 0o11);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);


    let bin = assemble(r#"
        movb 06, r0
        halt
        .word 0533
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o133);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);
}

#[test]
fn relative_def_read() {
    let bin = assemble(r#"
        .word 0410
        mov @00, r0
        halt
        .word 066
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START + 2);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o66);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);

    let bin = assemble(r#"
    label:
        .word 0410
        movb @00, r0
        halt
        .word 0533
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START + 2);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o133);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);
}

#[test]
fn relative_def_write() {
    let bin = assemble(r#"
        .word 0410
        mov #33, r0
        mov r0, @00
        halt
        .word 066
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START + 2);
    assert_eq!(emu.mem_read_word(0o410), 0o33);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);


    let bin = assemble(r#"
        .word 0414
        mov     #0, r0
        movb    r0, @00
        halt
        .word 07777
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START + 2);
    assert_eq!(emu.mem_read_word(0o414), 0o7400);
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16 - WORD_SIZE);
}

#[test]
fn cmp_literal_index() {
    let asm = r#"
        . = 400

    count:
        .word 0

    _start:
        cmp count, #9.
        halt
    "#;

    let (bin, symbols) = assemble_with_symbols(asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, 0);
    emu.run_at(*symbols.get("_start").unwrap());
    assert_eq!(emu.reg_read_word(Reg::PC), bin.len() as u16);


    let asm = r#"
        . = 400

    count:
        .word 0

    _start:
        cmp #9., count
        halt
    "#;

    let (bin, symbols) = assemble_with_symbols(asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, 0);
    emu.run_at(*symbols.get("_start").unwrap());
    assert_eq!(emu.reg_read_word(Reg::PC), bin.len() as u16);
}



