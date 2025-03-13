use as_lib::assemble_raw;
use common::asm::Reg;
use common::misc::ToU16P;
use emu_lib::Emulator;

// Assumes "proper" halt is last ins in binary
fn run(asm: &str) -> Emulator {
    let prog = assemble_raw(asm);
    let mut emu = Emulator::new();
    emu.load_image(&prog.text, 0);
    emu.run_at(prog.symbols.get("_start").unwrap().val);
    assert_eq!(emu.get_state().pc(), prog.text.len().to_u16p());
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
    assert_eq!(emu.reg_read_word(Reg::R0), 0o7);

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
    assert_eq!(emu.reg_read_word(Reg::R0), 0o4);
}

#[test]
fn emt_rti() {
    let asm = r#"
        STACK_TOP = 150000 

        . = 30
        .word handler, 0

        . = 400

    _start:
        mov #STACK_TOP, sp
        mov #5, r0
        emt
        br done

    handler:
        rti
        halt

    done:
        halt
    "#;
    let emu = run(asm);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o5);

    let asm = r#"
        STACK_TOP = 150000 

        . = 30
        .word handler, 0

        . = 400

    _start:
        mov #STACK_TOP, sp
        mov #1, r0
        clr r1
        emt 4
        mov ans, r1
        br done

    handler:
        mov r0, -(sp)
        mov 2(sp), r0   ; r0 = &old_pc
        sub #2, r0      ; r0 = &old_pc 
        mov (r0), r0    ; r0 = old_pc
        bic #177400, r0 ; r0 = old_pc & 0xff (r0 = payload)
        mov r0, ans
        mov (sp)+, r0
        rti
        halt

    ans:
        .word 0

    done:
        halt
    "#;
    let emu = run(asm);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o1);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o4);
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
    assert_eq!(emu.reg_read_word(Reg::R0), 0o7);

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
        mov 2(sp), r0   ; r0 = &old_pc
        sub #2, r0      ; r0 = &old_pc 
        mov (r0), r0    ; r0 = old_pc
        bic #177400, r0 ; r0 = old_pc & 0xff (r0 = payload)
        halt
    "#;
    let emu = run(asm);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o4);
}

#[test]
fn trap_rti() {
    let asm = r#"
        STACK_TOP = 150000 

        . = 34
        .word handler, 0

        . = 400

    _start:
        mov #STACK_TOP, sp
        mov #5, r0
        trap
        br done

    handler:
        rti
        halt

    done:
        halt
    "#;
    let emu = run(asm);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o5);

    let asm = r#"
        STACK_TOP = 150000 

        . = 34
        .word handler, 0

        . = 400

    _start:
        mov #STACK_TOP, sp
        mov #1, r0
        clr r1
        trap 4
        mov ans, r1
        br done

    handler:
        mov r0, -(sp)
        mov 2(sp), r0   ; r0 = &old_pc
        sub #2, r0      ; r0 = &old_pc 
        mov (r0), r0    ; r0 = old_pc
        bic #177400, r0 ; r0 = old_pc & 0xff (r0 = payload)
        mov r0, ans
        mov (sp)+, r0
        rti
        halt

    ans:
        .word 0

    done:
        halt
    "#;
    let emu = run(asm);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o1);
    assert_eq!(emu.reg_read_word(Reg::R1), 0o4);
}
