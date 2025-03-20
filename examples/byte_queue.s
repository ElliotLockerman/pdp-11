
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

    ; If full, return false.
    cmp     QUEUE_CAP(r0), QUEUE_LEN(r0)
    beq     3f

    ; Move r1 to buf[tail], increment len.
    mov     QUEUE_BUF(r0), r2
    add     QUEUE_TAIL(r0), r2
    movb    r1, (r2)
    inc     QUEUE_LEN(r0)

    ; Increment tail and wrap if needed.
    inc     QUEUE_TAIL(r0)
    cmp     QUEUE_CAP(r0), QUEUE_TAIL(r0)
    bne     1f

    clr     QUEUE_TAIL(r0)  ; Wrap tail

    ; Success, set return value to 1.
1:
    mov     #1, r0

    ; Return.
2:
    mov     (sp)+, r2
    rts     pc

    ; Full, set return value to 0.
3:
    mov     #0, r0
    br      2b


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn byte_queue_pop(r0 queue: &Queue) -> (r0 success: bool, r1 val: u8)
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
byte_queue_pop:
    ; If empty, return false
    tst     QUEUE_LEN(r0)
    beq     3f

    ; Move buf[head] to r1, decrement len
    mov     QUEUE_BUF(r0), r1
    add     QUEUE_HEAD(r0), r1
    movb    (r1), r1
    dec     QUEUE_LEN(r0)

    ; Increment head and wrap if needed
    inc     QUEUE_HEAD(r0)
    cmp     QUEUE_CAP(r0), QUEUE_HEAD(r0)
    bne     1f

    clr     QUEUE_HEAD(r0)  ; Wrap head

    ; Success, set return value to 1.
1:
    mov     #1, r0

    ; Return.
2:
    rts     pc

    ; Empty, set return value to 0.
3:
    mov     #0, r0
    br      2b


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


