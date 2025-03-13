use as_lib::assemble_raw;
use common::asm::Reg;
use common::misc::ToU16P;
use emu_lib::io::clock::{Clock, FakeClock};
use emu_lib::io::teletype::*;
use emu_lib::{Emulator, ExecRet};

use std::io::BufRead;
use std::sync::Arc;
use std::thread;

#[test]
fn hello() {
    let prog = assemble_raw(include_str!("../../examples/hello.s"));

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler_for(teletype, [Teletype::TPS, Teletype::TPB]);

    emu.load_image(&prog.text, 0);
    emu.run_at(prog.symbols.get("_start").unwrap().val);

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

    let prog = assemble_raw(asm);

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);
    emu.load_image(&prog.text, 0);
    emu.run_at(prog.symbols.get("_start").unwrap().val);

    let mut buf = tty.take_output();
    buf.make_contiguous();
    let out = String::from_utf8_lossy(buf.as_slices().0);
    assert_eq!(out, "hello, world!\n");
}

#[test]
fn clock() {
    let prog = assemble_raw(include_str!("../../examples/timer_ticks.s"));

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);
    emu.set_mmio_handler(Clock::default());
    emu.load_image(&prog.text, 0);
    emu.run_at(prog.symbols.get("_start").unwrap().val);

    let mut buf = tty.take_output();
    buf.make_contiguous();
    let out = String::from_utf8_lossy(buf.as_slices().0);
    assert_eq!(out, "123456789\n");
}

#[test]
fn fake_clock() {
    let times = 12;
    let asm = format!(
        r#"

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

    "#
    );
    let prog = assemble_raw(&asm);

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let clock = FakeClock::default();
    let striker = clock.get_striker();

    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);
    emu.set_mmio_handler(clock);
    emu.load_image(&prog.text, 0);
    emu.reg_write_word(Reg::SP, 0o150000);
    emu.mem_write_byte(FakeClock::LKS, 0x1 << FakeClock::INT_ENB_SHIFT);
    let start = prog.symbols.get("_start").unwrap().val;

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
    let prog = assemble_raw(include_str!("../../examples/threads.s"));

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);
    emu.set_mmio_handler(Clock::default());
    emu.load_image(&prog.text, 0);
    emu.reg_write_word(Reg::PC, prog.symbols.get("_start").unwrap().val);

    for _ in 0..250_000 {
        let ret = emu.run_ins();
        if ret == ExecRet::Halt {
            break;
        }
    }

    let mut buf = tty.take_output();
    buf.make_contiguous();
    let out = String::from_utf8_lossy(buf.as_slices().0);
    assert_eq!(out, "00000000000111111111110000000000111111111110000000");
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

    let prog = assemble_raw(&asm);

    let clock = FakeClock::default();
    let striker = clock.get_striker();

    let mut emu = Emulator::new();
    emu.set_mmio_handler(clock);
    striker.strike();
    emu.load_image(&prog.text, 0);
    emu.reg_write_word(Reg::PC, prog.symbols.get("_start").unwrap().val);

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

    let prog = assemble_raw(&asm);

    let clock = FakeClock::default();
    let striker = clock.get_striker();

    let mut emu = Emulator::new();
    emu.set_mmio_handler(clock);
    striker.strike();
    emu.load_image(&prog.text, 0);
    emu.reg_write_word(Reg::PC, prog.symbols.get("_start").unwrap().val);

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

    let prog = assemble_raw(&asm);

    let clock = FakeClock::default();
    let striker = clock.get_striker();

    let mut emu = Emulator::new();
    emu.set_mmio_handler(clock);
    striker.strike();
    emu.load_image(&prog.text, 0);
    emu.reg_write_word(Reg::PC, prog.symbols.get("_start").unwrap().val);

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

    let prog = assemble_raw(asm);

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);
    emu.load_image(&prog.text, 0);
    let val = 0o23u8;
    tty.push_input(val);
    emu.run_at(prog.symbols.get("_start").unwrap().val);
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

    let prog = assemble_raw(asm);

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);

    let input = b"Hello, world!\n";
    tty.write_input(input);
    emu.load_image(&prog.text, 0);
    emu.run_at(prog.symbols.get("_start").unwrap().val);

    let mut buf = tty.take_output();
    buf.make_contiguous();
    assert_eq!(&buf, input);
}

#[test]
fn pipe_echo_spin_line() {
    let asm = include_str!("../../examples/echo_spin.s");

    let prog = assemble_raw(asm);

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);

    const LINE_LIMIT: usize = 72;
    let lines: &[Vec<u8>] = &[
        b"sadlkfa".into(),
        b";la;lskdfjlaskfjds;lkfjas; dfkjs;lkdfja;slkfjaslkdfjas;ldkfjasl;dkfjal;kkjfasew".into(),
        b"aslfkdja;".into(),
    ];
    for line in lines {
        let len = usize::min(line.len(), LINE_LIMIT);
        let line = &line[0..len];
        tty.write_input(line);
        tty.push_input(b'\n');
    }
    emu.load_image(&prog.text, 0);

    emu.reg_write_word(Reg::PC, prog.symbols.get("_start").unwrap().val);
    for _ in 0..1_000_000 {
        let ret = emu.run_ins();
        if ret == ExecRet::Halt {
            break;
        }
    }

    let mut buf = tty.take_output();
    buf.make_contiguous();
    for (i, line) in buf.lines().enumerate() {
        let line = line.unwrap();
        let gold = &lines[i / 2];
        let len = usize::min(gold.len(), LINE_LIMIT);
        let gold = String::from_utf8_lossy(&gold[0..len]);
        assert_eq!(gold, line);
    }
}

#[test]
fn pipe_keyboard_interrupt() {
    let asm = r#"

        STACK_TOP = 150000 

        TPS = 177564
        TPB = TPS + 2
        TPS_READY_CMASK = 177
        TKS = 177560
        TKB = TKS + 2
        TKS_INT_ENB = 100

        PRIO7 = 340

        . = 60
        .word keyboard, PRIO7

        . = 400

    _start:
        mov #STACK_TOP, sp
        mov #TKS_INT_ENB, @#TKS

    loop:
        tst done
        beq loop

        halt

    done:
        .word 0

    keyboard:
        mov r0, -(sp)

        mov  @#TKB, r0
        movb r0, @next
        inc  next
        tst  r0
        bne  ret

        mov #1, done

    ret:
        mov (sp)+, r0
        rti

    buf:
    . = . + 100
    next:
        .word buf
    "#;

    let prog = assemble_raw(asm);

    let tty = Arc::new(PipeTty::default());
    let teletype = Teletype::new(tty.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler(teletype);
    emu.load_image(&prog.text, 0);
    let msg = b"foo bar baz\0";
    tty.write_input(msg);
    emu.run_at(prog.symbols.get("_start").unwrap().val);

    let buf = prog.symbols.get("buf").unwrap().val;
    for (i, ch) in msg.iter().enumerate() {
        assert_eq!(emu.mem_read_byte(buf + i.to_u16p()), *ch);
    }
}
