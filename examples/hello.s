    . = 400

    STACK_TOP = 150000 
    TPS = 177564
    TPB = TPS + 2
    TPS_READY_MASK = 177

_start:
    mov #STACK_TOP, sp
    mov #msg, r1

msg_loop:
    movb (r1)+, r0
    beq msg_loop_done
    jsr pc, print
    br msg_loop

msg_loop_done:
    movb #012, r0 ; '\n'
    jsr pc, print

    halt


; char to print in r0, others callee save
print:
    mov r1, -(sp)

print_loop:
    movb @#TPS, r1
    bicb #TPS_READY_MASK, r1
    beq print_loop

    movb r0, @#TPB
    mov (sp)+, r1
    rts pc  


msg:
.asciz "hello, world!"

