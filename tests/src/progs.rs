
use as_lib::{assemble, assemble_with_symbols};
use emu_lib::Emulator;
use common::asm::Reg;
use common::constants::DATA_START;

use std::assert_matches::assert_matches;

#[test]
fn looop() {
    let bin = assemble(r#"
        clr r0
    loop:
        inc r0
        cmp #12, r0
        bne loop

        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);
    emu.run_at(DATA_START);
    assert_eq!(emu.reg_read_word(Reg::R0), 0o12, "r0");
    assert_eq!(emu.reg_read_word(Reg::PC), DATA_START + bin.len() as u16);
}

#[test]
fn strcpy() {
    let bin = assemble(r#"
        br start
    out:
        . = . + 16
    in:
        .asciz "hello, world!"

        .even
    start:
        mov #in, r0
        mov #out, r1
        
    loop:
        cmpb #0, (r0)
        beq done

        movb (r0)+, (r1)+
        br loop

    done:
        clrb (r1)
        halt
    "#);
    let mut emu = Emulator::new();
    emu.load_image(&bin, 0);
    emu.run_at(0);

    let expected = b"  hello, world!\0";
    for byte_idx in 2u16..=15 {
        assert_eq!(
            emu.mem_read_byte(byte_idx),
            expected[byte_idx as usize]
        );
    }
}


#[test]
fn fib() {
    let bin = assemble(r#"
    br start

    out:
    .word 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
    out_end = .

        .even

    ; Arg and return in r0, rest callee save
    fib:
        cmp #0, r0
        beq done

        cmp #1, r0
        beq done

        mov r1, -(sp)
        mov r2, -(sp)
        mov r3, -(sp)

        dec r0
        mov r0, r1
        jsr pc, fib

        mov r0, r2
        mov r1, r0
        dec r0
        jsr pc, fib

        add r2, r0

        mov (sp)+, r3
        mov (sp)+, r2
        mov (sp)+, r1

    done:
        rts pc


    start:
        mov #150000, sp
        mov #0, r1
        mov #out, r3

    loop:
        cmp #out_end, r3
        beq done2

        mov r1, r0
        inc r1
        jsr pc, fib
        mov r0, (r3)+
        br loop

    done2:
        halt
    "#);


    let mut emu = Emulator::new();
    emu.load_image(&bin, 0);
    emu.run_at(0);

    fn fib(i: u16) -> u16 {
        match i {
            0 => 0,
            1 => 1,
            j => fib(j - 1) + fib(j - 2),
        }
    }

    for i in 0..10 {
        assert_eq!(emu.mem_read_word(i * 2 + 2), fib(i));
    }
}

#[test]
fn unsigned_mul() {
    let asm = r#"
        ; r0 x r1, lower result in r0, upper in r1
    _start:
        mov #17000, sp
        jsr pc, mulu
        halt

    mulu:
        mov r2, -(sp)
        mov r3, -(sp)
        mov r4, -(sp)
        mov r5, -(sp)

        mov r0, r2 ; operand being shifted left in (r2, r3)
        clr r0
        clr r3

        mov r1, r4 ; operand being shifted right in r4
        clr r1

    loop:
        tst r4
        beq done

        ; Check lowest bit of r5
        mov r4, r5
        bic #177776, r5
        cmp #1, r5
        bne cont

        ; If its set, add (r2, r3) to (r0, r1)
        add r2, r0
        adc r1
        add r3, r1

    cont:
        ; Shift r4 right _logically_
        clc
        ror r4

        ; Shift (r2, r3) left
        clc
        rol r2
        rol r3

        br loop

    done:
        mov (sp)+, r5
        mov (sp)+, r4
        mov (sp)+, r3
        mov (sp)+, r2
        rts pc
    "#;
    let bin = assemble(&asm);
    let mut emu = Emulator::new();
    emu.load_image(&bin, DATA_START);

    let mut run = |lhs, rhs| {
        emu.reg_write_word(Reg::R0, lhs);
        emu.reg_write_word(Reg::R1, rhs);
         
        emu.run_at(DATA_START);

        let lower = emu.reg_read_word(Reg::R0);
        let upper = emu.reg_read_word(Reg::R1);

        let gold = lhs.widening_mul(rhs);
        assert_eq!(lower, gold.0);
        assert_eq!(upper, gold.1);
    };

    run(0, 0);
    run(1, 0);
    run(0, 1);
    run(1, 1);
    run(2, 1);
    run(1, 2);
    run(5, 2);
    run(2, 5);
    run(u16::MAX, 0);
    run(0, u16::MAX);
    run(u16::MAX, 1);
    run(1, u16::MAX);
    run(u16::MAX, u16::MAX);
    run(u16::MAX, u16::MAX);

    let lhs: &[u16] = &[40165, 3211, 12898, 63636, 8366, 64413, 1698, 34815, 21398, 32909];
    let rhs: &[u16] = &[23273, 61041, 26275, 57783, 11729, 55426, 55264, 46246, 61796, 55239];
    for l in lhs {
        for r in rhs {
            run(*l, *r);
        }
    }
}

#[test]
fn byte_queue() {
    let queue = r#"

    ; Queue
    ; 0     buf: &u8    Underlying buffer.
    ; 2     head: u16   Index in to buf.
    ; 4     tail: u16   Index in to buf.
    ; 6     cap: u16    Length of buf in bytes.
    ; 10    len: u16    Number of elements in queue.

    QUEUE_BUF = 0 
    QUEUE_HEAD = 2
    QUEUE_TAIL = 4
    QUEUE_CAP = 6
    QUEUE_LEN = 10

    STATUS = 177776
    STATUS_Z_SHIFT = 177776 ; -1

    ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
    ; fn byte_queue_push(r0 queue: &Queue, r1 val: u8) -> r0 success: bool
    ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
    byte_queue_push:
        mov     r2, -(sp)

        ; If full, return false
        cmp     QUEUE_CAP(r0), QUEUE_LEN(r0)
        beq     byte_queue_push_full

        ; Move r1 to buf[tail], increment len
        mov     QUEUE_BUF(r0), r2
        add     QUEUE_TAIL(r0), r2
        movb    r1, (r2)
        inc     QUEUE_LEN(r0)

        ; Increment tail and wrap if needed
        inc     QUEUE_TAIL(r0)
        cmp     QUEUE_CAP(r0), QUEUE_TAIL(r0)
        bne     byte_queue_push_skip_wrap

        clr     QUEUE_TAIL(r0)  ; Wrap tail

    byte_queue_push_skip_wrap:
        mov     #1, r0

    byte_queue_push_done:
        mov     (sp)+, r2
        rts     pc

    byte_queue_push_full:
        mov     #0, r0
        br      byte_queue_push_done


    ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
    ; fn byte_queue_pop(r0 queue: &Queue) -> (r0 success: bool, r1 val: u8)
    ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
    byte_queue_pop:
        ; If empty, return false
        tst     QUEUE_LEN(r0)
        beq     byte_queue_pop_empty

        ; Move buf[head] to r1, decrement len
        mov     QUEUE_BUF(r0), r1
        add     QUEUE_HEAD(r0), r1
        movb    (r1), r1
        dec     QUEUE_LEN(r0)

        ; Increment head and wrap if needed
        inc     QUEUE_HEAD(r0)
        cmp     QUEUE_CAP(r0), QUEUE_HEAD(r0)
        bne     byte_queue_pop_skip_wrap

        clr     QUEUE_HEAD(r0)  ; Wrap head

    byte_queue_pop_skip_wrap:
        mov     #1, r0

    byte_queue_pop_done:
        rts     pc

    byte_queue_pop_empty:
        mov     #0, r0
        br      byte_queue_pop_done

    ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
    ; fn byte_queue_len(r0 queue: &Queue) -> r0 len: u16
    ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
    byte_queue_len:
        mov     QUEUE_LEN(r0), r0
        rts     pc

    ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
    ; fn byte_queue_full(r0 queue: &Queue) -> r0 full: bool
    ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
    byte_queue_full:
        cmp     QUEUE_CAP(r0), QUEUE_LEN(r0)
        mov     @#STATUS, r0
        ash     #STATUS_Z_SHIFT, r0
        bic     #177776, r0
        rts     pc

    ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
    ; fn byte_queue_empty(r0 queue: &Queue) -> r0 empty: bool
    ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
    byte_queue_empty:
        tst     QUEUE_LEN(r0)
        mov     @#STATUS, r0
        ash     #STATUS_Z_SHIFT, r0
        bic     #177776, r0
        rts     pc

    "#;

    let harness = r#"
        STACK_TOP = 150000 
        BUF_LEN = 20
        
        . = 400

    call_pop:
        mov     #STACK_TOP, sp
        mov     #queue, r0
        jsr     pc, byte_queue_pop
        halt
        
    call_push:
        mov     #STACK_TOP, sp
        mov     #queue, r0
        jsr     pc, byte_queue_push
        halt

    call_len:
        mov     #STACK_TOP, sp
        mov     #queue, r0
        jsr     pc, byte_queue_len
        halt

    call_full:
        mov     #STACK_TOP, sp
        mov     #queue, r0
        jsr     pc, byte_queue_full
        halt

    call_empty:
        mov     #STACK_TOP, sp
        mov     #queue, r0
        jsr     pc, byte_queue_empty
        halt

    queue:
        .word buf       ; buf
        .word 0         ; head
        .word 0         ; tail
        .word BUF_LEN   ; cap
        .word 0         ; len

    buf:
        . = . + BUF_LEN
    "#.to_owned();

    let (bin, symbols) = assemble_with_symbols(&(harness + queue));

    eprintln!("symbols: {:?}", symbols);
    let mut emu = Emulator::new();
    emu.load_image(&bin, 0);

    let push = |emu: &mut Emulator, val: u16| -> u16 {
        emu.reg_write_word(Reg::R1, val);
        emu.run_at(*symbols.get("call_push").unwrap());
        emu.reg_read_word(Reg::R0)
    };

    let pop = |emu: &mut Emulator| -> (u16, u16) {
        emu.run_at(*symbols.get("call_pop").unwrap());
        (
            emu.reg_read_word(Reg::R0),
            emu.reg_read_word(Reg::R1),
        )
    };

    let len = |emu: &mut Emulator| -> u16 {
        emu.run_at(*symbols.get("call_len").unwrap());
        emu.reg_read_word(Reg::R0)
    };

    let full = |emu: &mut Emulator| -> u16 {
        emu.run_at(*symbols.get("call_full").unwrap());
        emu.reg_read_word(Reg::R0)
    };

    let empty = |emu: &mut Emulator| -> u16 {
        emu.run_at(*symbols.get("call_empty").unwrap());
        emu.reg_read_word(Reg::R0)
    };

    assert_eq!(full(&mut emu), 0);
    assert_eq!(len(&mut emu), 0);
    assert_eq!(empty(&mut emu), 1);
    assert_matches!(pop(&mut emu), (0, _));

    assert_eq!(push(&mut emu, 27), 1);
    assert_eq!(full(&mut emu), 0);
    assert_eq!(len(&mut emu), 1);
    assert_eq!(empty(&mut emu), 0);
    assert_matches!(pop(&mut emu), (1, 27));

    let count = 5;
    for i in 1..=count {
        assert_eq!(push(&mut emu, i), 1);
        assert_eq!(full(&mut emu), 0);
        assert_eq!(len(&mut emu), i);
        assert_eq!(empty(&mut emu), 0);
    }

    for i in 1..=count {
        assert_eq!(pop(&mut emu), (1, i));
        assert_eq!(full(&mut emu), 0);
        assert_eq!(len(&mut emu), count - i);
        assert_eq!(empty(&mut emu), (i == count) as u16);
    }
    assert_eq!(len(&mut emu), 0);
    assert_eq!(empty(&mut emu), 1);
    assert_matches!(pop(&mut emu), (0, _));

    let count = *symbols.get("BUF_LEN").unwrap();
    for i in 1..=count {
        assert_eq!(push(&mut emu, i), 1);
        assert_eq!(full(&mut emu), (i == count) as u16);
        assert_eq!(len(&mut emu), i);
        assert_eq!(empty(&mut emu), 0);
    }

    assert_eq!(push(&mut emu, 1), 0);
    assert_eq!(pop(&mut emu), (1, 1));
    assert_eq!(full(&mut emu), 0);
    assert_eq!(len(&mut emu), count - 1);
    assert_eq!(empty(&mut emu), 0);

    assert_eq!(push(&mut emu, count + 1), 1);
    assert_eq!(push(&mut emu, 1), 0);
    assert_eq!(full(&mut emu), 1);
    assert_eq!(len(&mut emu), count);
    assert_eq!(empty(&mut emu), 0);

    for i in 2..=count + 1 {
        assert_eq!(pop(&mut emu), (1, i));
        assert_eq!(full(&mut emu), 0);
        assert_eq!(len(&mut emu), count + 1 - i);
        assert_eq!(empty(&mut emu), (i == count + 1) as u16);
    }

}

