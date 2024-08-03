
#[cfg(test)]
mod io {
    use std::sync::Arc;

    use as_lib::assemble;
    use emu_lib::Emulator;
    use emu_lib::io;

    #[test]
    fn hello() {
        let bin = assemble(r#"
            mov #0150000, sp
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
            mov r2, -(sp)
            mov r3, -(sp)
            mov #0177564, r2   ; TPS
            mov #0177566, r3   ; TPB

        print_loop:
            movb (r2), r1
            bicb #0177, r1 
            beq print_loop

            movb r0, (r3)

            mov (sp)+, r3
            mov (sp)+, r2
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
}

