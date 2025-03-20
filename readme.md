
# PDP-11 Emulator and Toolchain.

Its still early in development, but it can run some bare-metal program. Build them with `make -C examples`, and try e.g., `./emu examples/build/echo_interrupt` for an interrupt-based echo console with an emulated KH11 teletype.

## Repo Structure

- `aout/`: `a.out` object/binary format library.
- `assembler/`: PDP-11 assembler library and cli binary.
- `as`: script to compile and run assembler.
- `common/`: library with the assembly data structures, constants, and some helper functions.
- `disassembler/`: PDP-11 disassembler library and cli binary.
- `emulator/`: PDP-11 emulator library and cli binary.
- `emu`: script to compile and run emulator.
- `examples/`: example programs to run in the emulator.
    - `fib`: print the first 10 Fibonacci numbers.
    - `timer_ticks`: print at a fixed interval based on timer interrupts.
    - `echo_spin`: a spinning-based echo console.
    - `echo_interrupt`: an interrupt-based echo console.
    - `threads`: a demonstration of preemptive multitasking (two threads repeatedly print their TIDs).
- `interpreter/`: a helper cli binary that assembles and emulates the specified program.
- `interp`: script to compile and run interpreter.
- `tests/`: integration tests.


