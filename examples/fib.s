
    .even

_start:
    mov     #150000, sp
    mov     #0, r1

1:
    ; Call fib.
    mov     r1, r0
    jsr     pc, fib

    ; Print the result.
    jsr     pc, printu

    ; Print a newline.
    mov     #'\n, r0
    jsr     pc, putc

    ; Increment the induction variable and loop until it hits 10.
    inc     r1
    cmp     #10., r1
    bne     1b

    halt

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fib(r0 num: u16) -> u16
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
fib:
    ; Base case 1: fib(0) = 0.
    cmp     #0, r0
    beq     1f

    ; Base case 2: fib(1) = 1.
    cmp     #1, r0
    beq     1f

    ; Recursive case.
    ; Save variables we're using.
    mov     r1, -(sp)
    mov     r2, -(sp)

    ; fib(num - 1).
    dec     r0
    mov     r0, r1
    jsr     pc, fib

    ; fib(num - 2).
    mov     r0, r2  ; Save fib(num - 1) in r2.
    mov     r1, r0
    dec     r0
    jsr     pc, fib

    add     r2, r0

    mov     (sp)+, r2
    mov     (sp)+, r1

1:
    rts     pc

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn printu(r0 num: u16)
; Prints unsigned num in decimal.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
printu:
    mov     r1, -(sp)
    mov     r2, -(sp)

                    ; Lower half of 32-bit dividend in r0, the argument.
    clr     r1      ; Upper half of 32-bit dividend, which we're not using.
    mov     sp, r2  ; Save top of stack (one past last digit).

1:
    clr     r1          ; Upper half of 32-bit dividend, which we're not using.
    div     #10., r0    ; Quotient in r0, remainder in r1
    mov     r1, -(sp)   ; Save remainder.
    cmp     #0, r0      ; If quotient isn't 0, loop.
    bne     1b

    ; Now we have all the decimal digits on the stack in the range [sp, r2), and there must be at
    ; least one digit.
2:
    mov     (sp)+, r0   ; Pop a char.
    add     #48., r0    ; Convert to ascii.
    jsr     pc, putc   ; Print it.

    cmp     sp, r2      ; Not at the end?
    bne     2b          ; continue

    mov     (sp)+, r2
    mov     (sp)+, r1
    rts     pc









