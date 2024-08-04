
use std::sync::Arc;

use as_lib::assemble;
use emu_lib::Emulator;
use emu_lib::io;

#[test]
fn hello() {
    let bin = assemble(r#"
        STACK_TOP = 150000 
        TPS = 177564
        TPB = 177566
        TPS_READY_MASK = 177

        mov #STACK_TOP, sp
        mov #msg, r1

    msg_loop:
        movb (r1)+, r0
        beq msg_loop_done
        jsr pc, print
        br msg_loop

    msg_loop_done:
        movb #012, r0 ; '\n'
        jsr pc, print

        halt


    ; char to print in r0, others callee save
    print:
        mov r1, -(sp)

    print_loop:
        movb @#TPS, r1
        bicb #TPS_READY_MASK, r1
        beq print_loop

        movb r0, @#TPB
        mov (sp)+, r1
        rts pc  


    msg:
    .asciz "hello, world!"
    "#);


    let printer = Arc::new(io::PipePrinter::default());
    let teleprinter = io::Teleprinter::new(printer.clone());
    let mut emu = Emulator::new();
    emu.set_mmio_handler([io::Teleprinter::TPS, io::Teleprinter::TPB], teleprinter);
    emu.load_image(&bin, 0);
    emu.run();

    let mut buf = printer.take();
    buf.make_contiguous();
    let out = String::from_utf8_lossy(buf.as_slices().0);
    assert_eq!(out, "hello, world!\n");
}

