; hello.s
; Prints hello, world!\n

    . = 400

    STACK_TOP = 150000 

    TPS = 177564
    TPB = TPS + 2
    TPS_READY_MASK = 177

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn _start()
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
_start:
    mov #STACK_TOP, sp
    mov #msg, r1

    ; Get first char (we know there's at least one).
    movb (r1)+, r0

    ; loop over msg, printing each character
msg_loop:
    jsr pc, print

    ; Load next character, stopping when we reach \0.
    movb (r1)+, r0
    bne msg_loop

    ; Print the terminating newline
    movb #12, r0 ; '\n'
    jsr pc, print

    halt

msg:
.asciz "hello, world!"
    .even


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; print(char: r0)
; char is the ascii char to print.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
print:
    ; Loop until the teleprinter is ready to accept another character.
    bicb #TPS_READY_MASK, @#TPS
    beq print

    movb r0, @#TPB
    rts pc  


