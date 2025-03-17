
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

main_wait:
    wait
    
main_loop:
    ; Pop our new char from the keyboard queue (with interrupts disabled for synchronization).
    bis     #PRIO7, @#STATUS
    mov     #keyboard_queue, r0
    jsr     pc, byte_queue_pop
    bic     #PRIO7, @#STATUS

    ; If the queue was empty, wait for an interrupt.
    tst     r0
    beq     main_wait

    ; Check if the character was \n; it gets special handling.
    cmpb    #'\n, r1
    beq     main_nl

    ; If we got a character, check if we have room for it. If we already have a
    ; line-lengths worth in the queue, drop the character. ; We only echo
    ; characters we have room for, so the user will see that the character was
    ; dropped.
    mov     #line_queue, r0
    jsr     pc, byte_queue_len
    cmp     r0, #LINE_LEN
    bge     main_loop

    ; Echo the character and push for later.
    mov     r1, r0
    jsr     pc, printer_push
    mov     #line_queue, r0
    jsr     pc, byte_queue_push ; We checked the length, so this can't fail.

    br main_loop

main_nl:
    ; If the character was \n, we have room reserved, echo it, push it and print the line.
    movb    r1, r0
    jsr     pc, printer_push
    mov     #line_queue, r0
    jsr     pc, byte_queue_push

    mov     #line_queue, r0
    jsr     pc, printer_push_queue

    br main_loop



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
; fn print_push_queue(r0 queue: &Queue)
; Pop all elements from queue and push on print queue.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
printer_push_queue:
    mov r1, -(sp)
    mov r2, -(sp)

    mov     r0, r2

printer_push_queue_loop:
    mov     r2, r0
    mov     #line_queue, r0
    jsr     pc, byte_queue_pop
    tst     r0
    beq     printer_push_queue_done
    
    movb    r1, r0
    jsr     pc, printer_push
    br      printer_push_queue_loop


printer_push_queue_done:
    mov     (sp)+, r2
    mov     (sp)+, r1
    rts     pc



;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn printer_push(r0 char: u8)
; Enqueues character to be printed.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
printer_push:
    mov     r1, -(sp)
    movb    r0, r1

printer_push_loop:
    mov     #print_queue, r0
    bis     #PRIO7, @#STATUS
    jsr     pc, byte_queue_push 
    bic     #PRIO7, @#STATUS
    tst     r0
    bne     printer_push_done

    wait
    br      printer_push_loop

printer_push_done:
    ; The printer interrupt will disable interrupts once the queue is empty.
    ; enable interrupts in case that has occured to start up printing again.
    bis     #TPS_INT_ENB, @#TPS

    mov     (sp)+, r1
    rts     pc

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn printer()
; Received printer interrupt.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
printer:
    mov r0, -(sp)
    mov r1, -(sp)
    mov r2, -(sp)
    mov r3, -(sp)
    mov r4, -(sp)
    mov r5, -(sp)

    mov     #print_queue, r0
    jsr     pc, byte_queue_pop 
    tst     r0
    bne     printer_do_print

    ; print queue was empty. Disabled interrupts so when printer_enqueue is called,
    ; it can reenabled interrupts and get this called again.
    bic     #TPS_INT_ENB, @#TPS
    br      printer_done

printer_do_print:
    movb    r1, @#TPB

printer_done:
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
; fn print(r0 char: u8) -> r0
; Blocks until ready to print, then prints r0 and returns it.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; Blocks until ready to print; prints char passed r0 without modifying it.
print:
    ; Loop until the teletype is ready to accept another character.
    bicb #TPS_READY_CMASK, @#TPS
    beq  print

    movb r0, @#TPB
    rts  pc  


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn keyboard()
; Received keyboard interrupt.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
keyboard:
    mov r0, -(sp)
    mov r1, -(sp)
    mov r2, -(sp)
    mov r3, -(sp)
    mov r4, -(sp)
    mov r5, -(sp)

    movb    @#TKB, r1
    mov     #keyboard_queue, r0
    jsr     pc, byte_queue_push 
    ; Ignore failure, they're nothing to do, but that's why the queue is oversized.

keyboard_done:
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


    ; Queue
    ; 0     buf: &u8    Underlying buffer.
    ; 2     head: u16   Index in to buf.
    ; 4     tail: u16   Index in to buf.
    ; 6     cap: u16    Length of buf in bytes.
    ; 10    len: u16    Number of elements in queue.

    QUEUE_BUF = 0 
    QUEUE_HEAD = 2
    QUEUE_TAIL = 4
    QUEUE_CAP = 6
    QUEUE_LEN = 10

    STATUS_Z_SHIFT = 177776 ; -1

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn byte_queue_push(r0 queue: &Queue, r1 val: u8) -> r0 success: bool
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
byte_queue_push:
    mov     r2, -(sp)

    ; If full, return false
    cmp     QUEUE_CAP(r0), QUEUE_LEN(r0)
    beq     byte_queue_push_full

    ; Move r1 to buf[tail], increment len
    mov     QUEUE_BUF(r0), r2
    add     QUEUE_TAIL(r0), r2
    movb    r1, (r2)
    inc     QUEUE_LEN(r0)

    ; Increment tail and wrap if needed
    inc     QUEUE_TAIL(r0)
    cmp     QUEUE_CAP(r0), QUEUE_TAIL(r0)
    bne     byte_queue_push_skip_wrap

    clr     QUEUE_TAIL(r0)  ; Wrap tail

byte_queue_push_skip_wrap:
    mov     #1, r0

byte_queue_push_done:
    mov     (sp)+, r2
    rts     pc

byte_queue_push_full:
    mov     #0, r0
    br      byte_queue_push_done


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn byte_queue_pop(r0 queue: &Queue) -> (r0 success: bool, r1 val: u8)
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
byte_queue_pop:
    ; If empty, return false
    tst     QUEUE_LEN(r0)
    beq     byte_queue_pop_empty

    ; Move buf[head] to r1, decrement len
    mov     QUEUE_BUF(r0), r1
    add     QUEUE_HEAD(r0), r1
    movb    (r1), r1
    dec     QUEUE_LEN(r0)

    ; Increment head and wrap if needed
    inc     QUEUE_HEAD(r0)
    cmp     QUEUE_CAP(r0), QUEUE_HEAD(r0)
    bne     byte_queue_pop_skip_wrap

    clr     QUEUE_HEAD(r0)  ; Wrap head

byte_queue_pop_skip_wrap:
    mov     #1, r0

byte_queue_pop_done:
    rts     pc

byte_queue_pop_empty:
    mov     #0, r0
    br      byte_queue_pop_done

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn byte_queue_len(r0 queue: &Queue) -> r0 len: u16
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
byte_queue_len:
    mov     QUEUE_LEN(r0), r0
    rts     pc

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn byte_queue_full(r0 queue: &Queue) -> r0 full: bool
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
byte_queue_full:
    cmp     QUEUE_CAP(r0), QUEUE_LEN(r0)
    mov     @#STATUS, r0
    ash     #STATUS_Z_SHIFT, r0
    bic     #177776, r0
    rts     pc

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn byte_queue_empty(r0 queue: &Queue) -> r0 empty: bool
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
byte_queue_empty:
    tst     QUEUE_LEN(r0)
    mov     @#STATUS, r0
    ash     #STATUS_Z_SHIFT, r0
    bic     #177776, r0
    rts     pc


