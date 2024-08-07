
use std::sync::Arc;

use as_lib::assemble_with_symbols;
use emu_lib::Emulator;
use emu_lib::io::teleprinter::*;
use emu_lib::io::clock::Clock;

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
