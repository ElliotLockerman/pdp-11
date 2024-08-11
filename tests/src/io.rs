
use as_lib::assemble_with_symbols;
use emu_lib::Emulator;
use emu_lib::io::teleprinter::*;
use emu_lib::io::clock::{Clock, FakeClock};
use common::asm::Reg;

use std::sync::Arc;
use std::thread;

#[test]
fn hello() {
    let (bin, symbols) = assemble_with_symbols(include_str!("../../examples/hello.s"));

    let printer = Arc::new(PipePrinter::default());
    let teleprinter = Teleprinter::new(printer.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler([Teleprinter::TPS, Teleprinter::TPB], teleprinter);
    emu.load_image(&bin, 0);
    emu.run_at(*symbols.get("_start").unwrap());

    let mut buf = printer.take();
    buf.make_contiguous();
    let out = String::from_utf8_lossy(buf.as_slices().0);
    assert_eq!(out, "hello, world!\n");
}

#[test]
fn clock() {
    let (bin, symbols) = assemble_with_symbols(include_str!("../../examples/timer_ticks.s"));

    let printer = Arc::new(PipePrinter::default());
    let teleprinter = Teleprinter::new(printer.clone());
    let clock = Clock::default();
    let mut emu = Emulator::new();
    emu.set_mmio_handler([Teleprinter::TPS, Teleprinter::TPB], teleprinter);
    emu.set_mmio_handler([Clock::LKS], clock);
    emu.load_image(&bin, 0);
    emu.run_at(*symbols.get("_start").unwrap());

    let mut buf = printer.take();
    buf.make_contiguous();
    let out = String::from_utf8_lossy(buf.as_slices().0);
    assert_eq!(out, "123456789\n");
}

#[test]
fn fake_clock() {
    let times = 100;
    let asm = format!(r#"

        LKS = 177546

        TPS = 177564
        TPB = TPS + 2
        TPS_READY_MASK = 177

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

        mov LKS, r0 ; clear clock bit

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
        ; Loop until the teleprinter is ready to accept another character.
        bicb #TPS_READY_MASK, @#TPS
        beq print

        movb r0, @#TPB
        rts pc  

    "#);
    let (bin, symbols) = assemble_with_symbols(&asm);

    let printer = Arc::new(PipePrinter::default());
    let teleprinter = Teleprinter::new(printer.clone());
    let clock = FakeClock::default();
    let striker = clock.get_striker();

    let mut emu = Emulator::new();
    emu.set_mmio_handler([Teleprinter::TPS, Teleprinter::TPB], teleprinter);
    emu.set_mmio_handler([FakeClock::LKS], clock);
    emu.load_image(&bin, 0);
    emu.get_state_mut().reg_write_word(Reg::SP, 0o150000);
    emu.mem_write_byte(FakeClock::LKS, 0x1 << FakeClock::INT_ENB_SHIFT);
    let start = *symbols.get("_start").unwrap();

    let thread = thread::spawn(move || {
        emu.run_at(start);
    });

    assert!(printer.is_empty());
    for _ in 0..times {
        striker.strike();

        while printer.is_empty() {
            thread::yield_now(); 
        }
        let val = printer.pop_front().unwrap();
        assert_eq!(val, b'.');
        assert!(printer.is_empty());
    }

    thread.join().unwrap();
    assert!(printer.is_empty());
}
