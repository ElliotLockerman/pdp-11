; timer_ticks.s
; Prints digits 0 - 9 (one every 2^8 ticks), followed by a newline, then halts.

    STACK_TOP = 150000 

    LKS = 177546
    LKS_INT_ENB = 100

    TPS = 177564
    TPB = TPS + 2
    TPS_READY_MASK = 177

    . = 100
    .word clock, 300


    . = 400

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn _start()
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
_start:
    mov #STACK_TOP, sp
    mov #LKS_INT_ENB, @#LKS ; Enable clock interrupts.

    ; Just spin; the rest of the program happens in clock() in response to interrupts.
loop:
    wait
    br loop


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn clock()
; Handles clock interrupt. Every 2^8 ticks, prints count and increments it.
; After 9, prints \n and halts.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
clock:
    mov r0, -(sp)
    mov r1, -(sp)
    mov r2, -(sp)
    mov r3, -(sp)
    mov r4, -(sp)
    mov r5, -(sp)

    mov LKS, r0 ; clear clock bit

    ; Increment counter and print it.
    inc count
    mov count, r0
    add #'0, r0
    jsr pc, print

    ; If we haven't reached 9 yet, just return.
    cmp #9., count
    bgt done

    ; If we have just printed 9, print \n and halt.
    mov #12, r0 ; '\n'
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
    

    ; Counter to print.
count:
    .word 0




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

