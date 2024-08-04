
use as_lib::assemble;
use emu_lib::Emulator;
use common::asm::Reg;
use common::constants::DATA_START;

#[test]
fn test_loop() {
    let bin = assemble(r#"
        clr r0
    loop:
        inc r0
        cmp #12, r0
        bne loop

        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o12, "r0");
    assert_eq!(emu.get_state().reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn test_strcpy() {
    let bin = assemble(r#"
        br start
    out:
        .ascii "              "
    in:
        .asciz "hello, world!"

    start:
        mov #in, r0
        mov #out, r1
        
    loop:
        cmpb #0, (r0)
        beq done

        movb (r0)+, (r1)+
        br loop

    done:
        clrb (r1)
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, 0);
    emu.run_at(0);

    let expected = b"  hello, world!\0";
    for byte_idx in 2u16..=15 {
        assert_eq!(
            emu.get_state().mem_read_byte(byte_idx),
            expected[byte_idx as usize]
        );
    }
}


#[test]
fn test_fib() {
    let bin = assemble(r#"
    br start

    out:
    .word 0 0 0 0 0 0 0 0 0 0

    ; Arg and return in r0, rest callee save
    fib:
        cmp #0, r0
        beq done

        cmp #1, r0
        beq done

        mov r1, -(sp)
        mov r2, -(sp)
        mov r3, -(sp)

        dec r0
        mov r0, r1
        jsr pc, fib

        mov r0, r2
        mov r1, r0
        dec r0
        jsr pc, fib

        add r2, r0

        mov (sp)+, r3
        mov (sp)+, r2
        mov (sp)+, r1

    done:
        rts pc


    start:
        mov #150000, sp
        mov #0, r1
        mov #out, r3

    loop:
        cmp #12, r1
        beq done2

        mov r1, r0
        inc r1
        jsr pc, fib
        mov r0, (r3)+
        br loop

    done2:
        halt
    "#);


    let mut emu = Emulator::new();
    emu.load_image(&bin, 0);
    emu.run_at(0);

    fn fib(i: u16) -> u16 {
        match i {
            0 => 0,
            1 => 1,
            j => fib(j - 1) + fib(j - 2),
        }
    }

    for i in 0..10 {
        assert_eq!(emu.get_state().mem_read_word(i * 2 + 2), fib(i));
    }
}
