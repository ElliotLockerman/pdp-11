    STACK_TOP = 150000 
    LKS = 177546
    LKS_INT_ENB = 100
    TPS = 177564
    TPB = TPS + 2
    TPS_READY_MASK = 177

    . = 100
    .word clock, 300


    . = 400

_start:
    mov #STACK_TOP, sp
    mov #LKS_INT_ENB, @#LKS

loop:
    br loop

ticks:
    .word 0

count:
    .word 0

clock:
    mov r0, -(sp)
    mov r1, -(sp)
    mov r2, -(sp)
    mov r3, -(sp)
    mov r4, -(sp)
    mov r5, -(sp)


    inc ticks
    bne done

    inc count
    mov count, r0
    add #60, r0
    jsr pc, print

    mov count, r0
    cmp #9., r0
    bgt done

    mov #12, r0
    jsr pc, print
    halt

done:
    mov (sp)+, r5
    mov (sp)+, r4
    mov (sp)+, r3
    mov (sp)+, r2
    mov (sp)+, r1
    mov (sp)+, r0
    rti
    


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
