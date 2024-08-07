; threads.s
; Two threads run concurrently and print their thread id.

    STACK_0 = 150000 
    STACK_1 = 140000 

    LKS = 177546
    LKS_INT_ENB = 100
    TPS = 177564
    TPB = TPS + 2
    TPS_READY_MASK = 177

    . = 100
    .word clock, 300 ; Clock interrupt vector

    . = 400

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn _start()
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
_start:
    ; Set up thread 1 for a return from clock().
    mov #STACK_1, sp
    mov #0, -(sp)    ; dummy return address
    mov #0, -(sp)    ; ps
    mov #run, -(sp)  ; pc
    mov #'1, -(sp)   ; r0: tid (as char) to print
    mov #0, -(sp)    ; r1
    mov #0, -(sp)    ; r2
    mov #0, -(sp)    ; r3
    mov #0, -(sp)    ; r4
    mov #0, -(sp)    ; r5
    mov sp, tcb_1
    
    ; Set up thread 0; it will be jumped to directly when it starts rather than
    ; returning from clock().
    mov #STACK_0, sp
    mov #0, -(sp) ; Dummy return address.
    mov #'0, r0   ; Tid (as char) to print.

    mov #LKS_INT_ENB, @#LKS ; Enable clock interrupts.

    ; Launch thread 0 (never returns).
    br run


    ; Index of currently running thread in tcb array.
tid:
    .word 0

    ; Thread Control Block (tcb) array.
tcbs:
tcb_0:
    .word 0 ; saved sp
tcb_1:
    .word 0 ; saved sp




;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn run(tid: r0)
; Main function for each thread - loop and print thread id every 2^16 iterations.
; tid is the thread id in its ascii digit.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
run:
    ; loop counter in r1.
    clr r1

    ; loop forever, printing tid once every 2^16 iterations.
run_loop:
    inc r1
    cmp #0, r1
    bne run_loop

    jsr pc, print
    br run_loop
    




;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn clock()
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
clock:
    mov r0, -(sp)
    mov r1, -(sp)
    mov r2, -(sp)
    mov r3, -(sp)
    mov r4, -(sp)
    mov r5, -(sp)

    mov LKS, r0 ; clear clock bit

    ; Increment tick counter; if it hasn't rolled over, just return.
    incb ticks
    cmpb #0, ticks
    bne clock_done

    ; Every time the tick counter rolls over, swap threads.

    ; Save currently running thread's sp
    mov tid, r0
    asl r0 ; index *= sizeof(tcb)
    mov sp, tcbs(r0)

    ; Toggle tid between 0 and 1
    inc tid
    bic #177776, tid

    ; Restore sp of new thread
    mov tid, r0
    asl r0 ; index *= sizeof(tcb)
    mov tcbs(r0), sp

clock_done:
    mov (sp)+, r5
    mov (sp)+, r4
    mov (sp)+, r3
    mov (sp)+, r2
    mov (sp)+, r1
    mov (sp)+, r0

    rti


    ; Total number of timer ticks, wrapping.
ticks:
    .word 0


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; print(char)
; char to print in r0
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
print:
    ; Loop until the teleprinter is ready to accept another character.
    bicb #TPS_READY_MASK, @#TPS
    beq print

    movb r0, @#TPB
    rts pc  

