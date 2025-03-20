
    STACK_TOP = 150000 

    TPS = 177564
    TPB = TPS + 2
    TPS_READY_CMASK = 177
    TPS_INT_ENB = 100
    TKS = 177560
    TKB = TKS + 2
    TKS_DONE_CMASK = 177577
    TKS_INT_ENB = 100

    LINE_LEN = 72.
    KEYBOARD_BUF_LEN = LINE_LEN + 1
    LINE_BUF_LEN = LINE_LEN + 1 ; line + \n
    PRINT_BUF_LEN = LINE_LEN + 1

    STATUS = 177776
    PRIO7 = 340

    . = 60
    .word keyboard, PRIO7
    .word printer, PRIO7

    . = 400

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn _start()
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
_start:
    mov     #STACK_TOP, sp
    bis     #TKS_INT_ENB, @#TKS
    bis     #TPS_INT_ENB, @#TPS

1:
    wait
    
2:
    ; Pop our new char from the keyboard queue (with interrupts disabled for synchronization).
    bis     #PRIO7, @#STATUS
    mov     #keyboard_queue, r0
    jsr     pc, byte_queue_pop
    bic     #PRIO7, @#STATUS

    ; If the queue was empty, wait for an interrupt.
    tst     r0
    beq     1b

    ; Check if the character was \n; it gets special handling.
    cmpb    #'\n, r1
    beq     3f

    ; If we got a character, check if we have room for it. If we already have a
    ; line-lengths worth in the queue, drop the character. ; We only echo
    ; characters we have room for, so the user will see that the character was
    ; dropped.
    mov     #line_queue, r0
    jsr     pc, byte_queue_len
    cmp     r0, #LINE_LEN
    bge     1b

    ; Echo the character and push for later.
    mov     r1, r0
    jsr     pc, printer_push
    mov     #line_queue, r0
    jsr     pc, byte_queue_push ; We checked the length, so this can't fail.

    br 2b

3:
    ; If the character was \n, we have room reserved, echo it, push it and print the line.
    movb    r1, r0
    jsr     pc, printer_push
    mov     #line_queue, r0
    jsr     pc, byte_queue_push

    mov     #line_queue, r0
    jsr     pc, printer_push_queue

    br 2b



line_queue:
    .word line_buf      ; buf
    .word 0             ; head
    .word 0             ; tail
    .word LINE_BUF_LEN  ; cap
    .word 0             ; len

line_buf:
    . = . + LINE_BUF_LEN

.even


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn print_push_queue()
; Pop all elements from line queue and push on print queue.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
printer_push_queue:
    mov r1, -(sp)

1:
    mov     #line_queue, r0
    jsr     pc, byte_queue_pop
    tst     r0
    beq     2f
    
    movb    r1, r0
    jsr     pc, printer_push
    br      1b


2:
    mov     (sp)+, r1
    rts     pc



;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn printer_push(r0 char: u8)
; Enqueues character to be printed, waiting if full.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
printer_push:
    mov     r1, -(sp)
    movb    r0, r1

1:
    mov     #print_queue, r0
    bis     #PRIO7, @#STATUS
    jsr     pc, byte_queue_push 
    bic     #PRIO7, @#STATUS
    tst     r0
    bne     2f

    ; Its full. Wait for an interrupt and try again.
    wait
    br      1b

2:
    ; The printer interrupt will disable interrupts once the queue is empty.
    ; enable interrupts in case that has occured to start up printing again.
    bis     #TPS_INT_ENB, @#TPS

    mov     (sp)+, r1
    rts     pc


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn printer()
; Received printer interrupt, printing a character from print_queue (if present).
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
printer:
    mov r0, -(sp)
    mov r1, -(sp)
    mov r2, -(sp)
    mov r3, -(sp)
    mov r4, -(sp)
    mov r5, -(sp)

    ; Pop a character to print.
    mov     #print_queue, r0
    jsr     pc, byte_queue_pop 
    tst     r0
    bne     1f

    ; print queue was empty. Disabled interrupts so when printer_push is called,
    ; it can reenabled interrupts and get this called again.
    bic     #TPS_INT_ENB, @#TPS
    br      2f

1:
    ; We go a character.
    movb    r1, @#TPB

2:
    mov (sp)+, r5
    mov (sp)+, r4
    mov (sp)+, r3
    mov (sp)+, r2
    mov (sp)+, r1
    mov (sp)+, r0
    rti


print_queue:
    .word print_buf     ; buf
    .word 0             ; head
    .word 0             ; tail
    .word PRINT_BUF_LEN ; cap
    .word 0             ; len

print_buf:
    . = . + PRINT_BUF_LEN

.even


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn keyboard()
; Received keyboard interrupt, push new character on to keyboard_queue.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
keyboard:
    mov r0, -(sp)
    mov r1, -(sp)
    mov r2, -(sp)
    mov r3, -(sp)
    mov r4, -(sp)
    mov r5, -(sp)

    ; Read character and push on to queue.
    movb    @#TKB, r1
    mov     #keyboard_queue, r0
    jsr     pc, byte_queue_push 
    ; Ignore failure, they're nothing to do, but that's why the queue is oversized.

    mov (sp)+, r5
    mov (sp)+, r4
    mov (sp)+, r3
    mov (sp)+, r2
    mov (sp)+, r1
    mov (sp)+, r0
    rti


keyboard_queue:
    .word keyboard_buf      ; buf
    .word 0                 ; head
    .word 0                 ; tail
    .word KEYBOARD_BUF_LEN  ; cap
    .word 0                 ; len

keyboard_buf:
    . = . + KEYBOARD_BUF_LEN

.even

