; hello.s
; Prints hello, world!\n

    . = 64
    .word tp_ready, 200

    . = 400

    STACK_TOP = 150000 

    TPS = 177564
    TPB = TPS + 2
    TPS_READY_MASK = 177
    TPS_INT_ENB = 100

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn _start()
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
_start:
    mov #STACK_TOP, sp
    mov #TPS_INT_ENB, @#TPS

loop:
    wait
    br loop

msg:
.ascii "hello, world!"
.byte '\n, '\0
    .even

next:
    .word msg

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; tp_ready()
; Teleprinter ready to accept another character; print it!
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
tp_ready:
    mov r0, -(sp)
    mov r1, -(sp)
    mov r2, -(sp)
    mov r3, -(sp)
    mov r4, -(sp)
    mov r5, -(sp)

    movb @next, r0
    inc next

    cmp #0, r0
    bne cont
    halt

cont:
    movb r0, @#TPB

    mov (sp)+, r5
    mov (sp)+, r4
    mov (sp)+, r3
    mov (sp)+, r2
    mov (sp)+, r1
    mov (sp)+, r0
    rti




