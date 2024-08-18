
use as_lib::assemble_with_symbols;
use emu_lib::{Emulator, ExecRet};
use emu_lib::io::teletype::*;
use emu_lib::io::clock::{Clock, FakeClock};
use common::asm::Reg;

use std::sync::Arc;
use std::thread;

#[test]
fn hello() {
    let (bin, symbols) = assemble_with_symbols(include_str!("../../examples/hello.s"));

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler_for(teletype, [Teletype::TPS, Teletype::TPB]);

    emu.load_image(&bin, 0);
    emu.run_at(*symbols.get("_start").unwrap());

    let mut buf = tty.take_output();
    buf.make_contiguous();
    let out = String::from_utf8_lossy(buf.as_slices().0);
    assert_eq!(out, "hello, world!\n");
}

#[test]
fn hello_spin() {
    let asm = r#"
        . = 400

        STACK_TOP = 150000 

        TPS = 177564
        TPB = TPS + 2
        TPS_READY_CMASK = 177

    _start:
        mov #STACK_TOP, sp
        mov #msg, r1

        ; Get first char (we know there's at least one).
        movb (r1)+, r0

        ; loop over msg, printing each character
    msg_loop:
        jsr pc, print

        ; Load next character, stopping when we reach \0.
        movb (r1)+, r0
        bne msg_loop

        ; Print the terminating newline
        movb #12, r0 ; '\n'
        jsr pc, print

        halt

    msg:
    .asciz "hello, world!"
        .even

    print:
        ; Loop until the teletype is ready to accept another character.
        bicb #TPS_READY_CMASK, @#TPS
        beq print

        movb r0, @#TPB
        rts pc  
    "#;

    let (bin, symbols) = assemble_with_symbols(asm);

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);
    emu.load_image(&bin, 0);
    emu.run_at(*symbols.get("_start").unwrap());

    let mut buf = tty.take_output();
    buf.make_contiguous();
    let out = String::from_utf8_lossy(buf.as_slices().0);
    assert_eq!(out, "hello, world!\n");
}

#[test]
fn clock() {
    let (bin, symbols) = assemble_with_symbols(include_str!("../../examples/timer_ticks.s"));

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);
    emu.set_mmio_handler(Clock::default());
    emu.load_image(&bin, 0);
    emu.run_at(*symbols.get("_start").unwrap());

    let mut buf = tty.take_output();
    buf.make_contiguous();
    let out = String::from_utf8_lossy(buf.as_slices().0);
    assert_eq!(out, "123456789\n");
}

#[test]
fn fake_clock() {
    let times = 12;
    let asm = format!(r#"

        LKS = 177546

        TPS = 177564
        TPB = TPS + 2
        TPS_READY_CMASK = 177

        . = 100
        .word clock, 300


        . = 400

    _start:
        ; Just spin; the rest of the program happens in clock() in response to interrupts.
    loop:
        wait
        br loop


    clock:
        mov r0, -(sp)
        mov r1, -(sp)
        mov r2, -(sp)
        mov r3, -(sp)
        mov r4, -(sp)
        mov r5, -(sp)

        mov @#LKS, r0 ; clear clock bit

        mov #'., r0
        jsr pc, print

        ; If we haven't reached the number of times yet, just return.
        inc count
        cmp #{times}., count
        bgt done

        ; If we have reached the number of times, halt.
        halt

    done:
        mov (sp)+, r5
        mov (sp)+, r4
        mov (sp)+, r3
        mov (sp)+, r2
        mov (sp)+, r1
        mov (sp)+, r0
        rti

    count:
        .word 0


    print:
        ; Loop until the teletype is ready to accept another character.
        bicb #TPS_READY_CMASK, @#TPS
        beq print

        movb r0, @#TPB
        rts pc  

    "#);
    let (bin, symbols) = assemble_with_symbols(&asm);

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let clock = FakeClock::default();
    let striker = clock.get_striker();

    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);
    emu.set_mmio_handler(clock);
    emu.load_image(&bin, 0);
    emu.reg_write_word(Reg::SP, 0o150000);
    emu.mem_write_byte(FakeClock::LKS, 0x1 << FakeClock::INT_ENB_SHIFT);
    let start = *symbols.get("_start").unwrap();

    let thread = thread::spawn(move || {
        emu.run_at(start);
    });

    assert!(tty.is_out_empty());
    for _ in 0..times {
        striker.strike();

        while tty.is_out_empty() {
            thread::yield_now(); 
        }
        let val = tty.pop_output().unwrap();
        assert_eq!(val, b'.');
        assert!(tty.is_out_empty());
    }

    thread.join().unwrap();
    assert!(tty.is_out_empty());
}

#[test]
fn threads() {
    let (bin, symbols) = assemble_with_symbols(include_str!("../../examples/threads.s"));

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);
    emu.set_mmio_handler(Clock::default());
    emu.load_image(&bin, 0);
    emu.reg_write_word(Reg::PC, *symbols.get("_start").unwrap());

    for _ in 0..2_000_000 {
        let ret = emu.run_ins();
        if ret == ExecRet::Halt {
            break;
        }
    }

    let mut buf = tty.take_output();
    buf.make_contiguous();
    let out = String::from_utf8_lossy(buf.as_slices().0);
    assert_eq!(out, "000011110");
}

#[test]
fn prio() {
    // Check that we get regular interrupts.
    let asm = r#"
        LKS = 177546
        LKS_INT_ENB = 100
        STACK_TOP = 150000

        . = 100
        .word clock, 300

        . = 400
    _start:
        mov #STACK_TOP, sp
        clr r0
        clr r1
        mov #LKS_INT_ENB, @#LKS

    loop:
        inc r1
        cmp #10., r1
        bne loop
        halt

    clock:
        mov #3, r0
        halt
    "#;

    let (bin, symbols) = assemble_with_symbols(&asm);

    let clock = FakeClock::default();
    let striker = clock.get_striker();

    let mut emu = Emulator::new();
    emu.set_mmio_handler(clock);
    striker.strike();
    emu.load_image(&bin, 0);
    emu.reg_write_word(Reg::PC, *symbols.get("_start").unwrap());

    striker.strike();
    emu.run();

    assert_eq!(emu.reg_read_word(Reg::R0), 3);


    // Check that we don't get interrupts when we raise the priority.
    let asm = r#"
        LKS = 177546
        LKS_INT_ENB = 100
        STACK_TOP = 150000
        STATUS = 177776
        PRIO7 = 340

        . = 100
        .word clock, 300

        . = 400
    _start:
        mov #STACK_TOP, sp
        clr r0
        clr r1
        bis #PRIO7, @#STATUS
        mov #LKS_INT_ENB, @#LKS

    loop:
        inc r1
        cmp #10., r1
        bne loop
        halt

    clock:
        mov #3, r0
        halt
    "#;

    let (bin, symbols) = assemble_with_symbols(&asm);

    let clock = FakeClock::default();
    let striker = clock.get_striker();

    let mut emu = Emulator::new();
    emu.set_mmio_handler(clock);
    striker.strike();
    emu.load_image(&bin, 0);
    emu.reg_write_word(Reg::PC, *symbols.get("_start").unwrap());

    striker.strike();
    emu.run();

    assert_eq!(emu.reg_read_word(Reg::R0), 0);


    // Check that we get interrupts again once we lower the priority.
    let asm = r#"
        LKS = 177546
        LKS_INT_ENB = 100
        STACK_TOP = 150000
        STATUS = 177776
        PRIO7 = 340

        . = 100
        .word clock, 300

    _start:
        mov #STACK_TOP, sp
        clr r0
        clr r1
        bis #PRIO7, @#STATUS
        mov #LKS_INT_ENB, @#LKS

    loop:
        inc r1
        cmp #10., r1
        bne loop

        bic #PRIO7, @#STATUS

    loop2:
        inc r1
        cmp #20., r1
        bne loop2
        halt

    clock:
        mov #3, r0
        halt
    "#;

    let (bin, symbols) = assemble_with_symbols(&asm);

    let clock = FakeClock::default();
    let striker = clock.get_striker();

    let mut emu = Emulator::new();
    emu.set_mmio_handler(clock);
    striker.strike();
    emu.load_image(&bin, 0);
    emu.reg_write_word(Reg::PC, *symbols.get("_start").unwrap());

    striker.strike();
    emu.run();

    assert_eq!(emu.reg_read_word(Reg::R0), 3);
    assert_eq!(emu.reg_read_word(Reg::R1), 10);
}

#[test]
fn pipe_keyboard_spin() {
    let asm = r#"
        TKS = 177560
        TKB = TKS + 2
        TKS_DONE_CMASK = 177577

        . = 400

    _start:
        bic #TKS_DONE_CMASK, @#TKS
        beq _start

        movb @#TKB, r0
        halt
    "#;

    let (bin, symbols) = assemble_with_symbols(asm);

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);
    emu.load_image(&bin, 0);
    let val = 0o23u8;
    tty.push_input(val);
    emu.run_at(*symbols.get("_start").unwrap());
    assert_eq!(emu.reg_read_byte(Reg::R0), val);
}

#[test]
fn pipe_echo_spin() {
    let asm = r#"
        STACK_TOP = 150000 

        TPS = 177564
        TPB = TPS + 2
        TPS_READY_CMASK = 177
        TKS = 177560
        TKB = TKS + 2
        TKS_DONE_CMASK = 177577

        . = 400

    _start:
        mov #STACK_TOP, sp

    read_loop:
        jsr  pc, read
        jsr  pc, print
        cmpb #'\n, r0
        bne  read_loop

        halt

    read:
        bic #TKS_DONE_CMASK, @#TKS
        beq read

        movb @#TKB, r0
        rts  pc

    print:
        ; Loop until the teletype is ready to accept another character.
        bicb #TPS_READY_CMASK, @#TPS
        beq  print

        movb r0, @#TPB
        rts  pc  
    "#;

    let (bin, symbols) = assemble_with_symbols(asm);

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);
    
    let input = b"Hello, world!\n";
    tty.write_input(input);
    emu.load_image(&bin, 0);
    emu.run_at(*symbols.get("_start").unwrap());

    let mut buf = tty.take_output();
    buf.make_contiguous();
    assert_eq!(&buf, input);
}
