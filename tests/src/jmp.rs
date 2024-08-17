use as_lib::assemble;
use emu_lib::Emulator;
use common::asm::Reg;
use common::constants::DATA_START;

// asm must set r0 to 1 if the jsr wasn't successful, 2 if it was
fn run(asm: &str) {
    let bin = assemble(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    println!("pc: {:o}", emu.reg_read_word(Reg::PC));
    assert_eq!(emu.reg_read_word(Reg::R0), 2);
}

#[test]
fn label() {
    run(r#"
        jmp taken

        mov #1, r0
        halt

    taken:
        mov #2, r0
        halt
        
    "#);
}

#[test]
fn relative() {
    run(r#"
        jmp 12

        mov #1, r0
        halt

        mov #2, r0
        halt
    "#);

    run(r#"
        br start

        mov #1, r0
        halt

    start:
        jmp 14

        mov #2, r0
        halt
    "#);
}

#[test]
fn abs() {
    run(r#"
        jmp @#412

        mov #1, r0
        halt

        mov #2, r0
        halt
    "#);
}


#[test]
fn def() {
    run(r#"
        mov #414, r1
        jmp (r1)

        mov #1, r0
        halt

        mov #2, r0
        halt
    "#);
}

#[test]
#[should_panic]
fn reg() {
    run(r#"
        jmp r1
    "#);
}
