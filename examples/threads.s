    STACK_0 = 150000 
    STACK_1 = 140000 

    LKS = 177546
    LKS_INT_ENB = 100
    TPS = 177564
    TPB = TPS + 2
    TPS_READY_MASK = 177

    . = 100
    .word clock, 300

    . = 400

current_thread:
    .word 0

tcbs:
tcb_0:
    .word 0 ; sp
tcb_1:
    .word 0 ; sp

_start:

    ; set up thread 1
    mov #STACK_1, sp
    mov #0, -(sp)    ; dummy return address

    mov #0, -(sp)    ; ps
    mov #run, -(sp)  ; pc
    mov #61, -(sp)   ; r0 - char to print
    mov #0, -(sp)    ; r1 - iteration
    mov #0, -(sp)    ; r2 - unused
    mov #0, -(sp)    ; r3 - unused
    mov #0, -(sp)    ; r4 - unused
    mov #0, -(sp)    ; r5 - unused
    mov sp, tcb_1
    
    ; set up thread 0
    mov #STACK_0, sp
    mov #0, -(sp) ; dummy return address
    mov #60, r0    ; char to print
    clr r1        ; iteration

    mov #LKS_INT_ENB, @#LKS

    br run


ticks:
    .word 0

clock:
    mov r0, -(sp)
    mov r1, -(sp)
    mov r2, -(sp)
    mov r3, -(sp)
    mov r4, -(sp)
    mov r5, -(sp)

    mov LKS, r0 ; clear clock bit

    incb ticks
    cmpb #0, ticks
    bne clock_done

    mov current_thread, r0
    mov r0, r1
    asl r1 ; index in to tcbs
    add #tcbs, r1
    mov sp, (r1)

    inc r0
    bic #177776, r0
    mov r0, current_thread

    asl r0 ; index in to tcbs
    add #tcbs, r0
    mov (r0), sp

clock_done:
    mov (sp)+, r5
    mov (sp)+, r4
    mov (sp)+, r3
    mov (sp)+, r2
    mov (sp)+, r1
    mov (sp)+, r0

    rti


run:
    inc r1
    cmp #0, r1
    bne run

    jsr pc, print
    br run
    

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
