
# PDP-11 Emulator and Toolchain.

Its still early in development, but it can run some bare-metal program. Try `./interp examples/echo_interrupt.s` for an interrupt-based echo console or `./interp examples/threads.s` for a demonstration of preemptive multitasking (the 0s and 1s are printed by different threads).


## Repo Structure

- `aout`: `a.out` object/binary format library.
- `assembler`: PDP-11 assembler library and cli binary.
- `common`: library with the assembly data structures, constants, and some helper functions.
- `disassembler`: PDP-11 disassembler library and cli binary.
- `emulator`: PDP-11 emulator library and cli binary.
- `examples`: example programs to run in the emulator.
- `interpreter`: a helper cli binary that assembles and emulates the specified program.
- `tests`: integration tests.

`as`, `disass`, `emu`, and `interp` are scripts to compile and run the assembler, disassembler, emulator, and interpreter binaries (respectively).

