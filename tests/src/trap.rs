
use as_lib::assemble_with_symbols;
use emu_lib::Emulator;
use common::asm::Reg;

// Assumes "proper" halt is last ins in binary
fn run(asm: &str) -> Emulator {
    let (bin, sym) = assemble_with_symbols(asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, 0);
    emu.run_at(*sym.get("_start").unwrap());
    assert_eq!(emu.get_state().pc(), bin.len() as u16);
    emu
}

#[test]
fn emt() {
    let asm = r#"
        STACK_TOP = 150000 

        . = 30
        .word handler, 0

        . = 400

    _start:
        mov #STACK_TOP, sp
        emt
        halt

    handler:
        mov #7, r0
        halt
    "#;

    let emu = run(asm);
    assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o7);

    let asm = r#"
        STACK_TOP = 150000 

        . = 30
        .word handler, 0

        . = 400

    _start:
        mov #STACK_TOP, sp
        emt 4
        halt

    handler:
        mov r0, -(sp)
        mov 2(sp), r0    ; r0 = &old_pc + 2
        sub #2, r0      ; r0 = &old_pc 
        mov (r0), r0    ; r0 = old_pc
        bic #177400, r0 ; r0 = old_pc & 0xff (payload)
        halt
    "#;
    let emu = run(asm);
    assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o4);
}


#[test]
fn trap() {
    let asm = r#"
        STACK_TOP = 150000 

        . = 34
        .word handler, 0

        . = 400

    _start:
        mov #STACK_TOP, sp
        trap
        halt

    handler:
        mov #7, r0
        halt
    "#;

    let emu = run(asm);
    assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o7);

    let asm = r#"
        STACK_TOP = 150000 

        . = 34
        .word handler, 0

        . = 400

    _start:
        mov #STACK_TOP, sp
        trap 2 + 2
        halt

    handler:
        mov r0, -(sp)
        mov 2(sp), r0    ; r0 = &old_pc + 2
        sub #2, r0      ; r0 = &old_pc 
        mov (r0), r0    ; r0 = old_pc
        bic #177400, r0 ; r0 = old_pc & 0xff (payload)
        halt
    "#;
    let emu = run(asm);
    assert_eq!(emu.get_state().reg_read_word(Reg::R0), 0o4);
}


