
    TPS = 177564
    TPB = TPS + 2
    TPS_READY_CMASK = 177

    .even


_start:
    mov     #150000, sp
    mov     #0, r1

1:
    mov     r1, r0
    inc     r1
    jsr     pc, fib

    jsr     pc, printu

    mov     #'\n, r0
    jsr     pc, print

    cmp     #10., r1
    bne     1b

    halt

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fib(r0 num: u16) -> u16
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
fib:
    cmp     #0, r0
    beq     1f

    cmp     #1, r0
    beq     1f

    mov     r1, -(sp)
    mov     r2, -(sp)
    mov     r3, -(sp)

    dec     r0
    mov     r0, r1
    jsr     pc, fib

    mov     r0, r2
    mov     r1, r0
    dec     r0
    jsr     pc, fib

    add     r2, r0

    mov     (sp)+, r3
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
    jsr     pc, print   ; Print it.

    cmp     sp, r2      ; Not at the end?
    bne     2b          ; continue

    mov     (sp)+, r2
    mov     (sp)+, r1
    rts     pc









;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn print(r0 char: u8)
; Blocks until ready to print, then prints r0.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
print:
    ; Loop until the teletype is ready to accept another character.
    bicb    #TPS_READY_CMASK, @#TPS
    beq     print

    movb    r0, @#TPB
    rts     pc  
