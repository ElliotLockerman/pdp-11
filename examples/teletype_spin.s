
    TPS = 177564
    TPB = TPS + 2
    TPS_READY_CMASK = 177
    TKS = 177560
    TKB = TKS + 2
    TKS_DONE_CMASK = 177577


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn putc(r0 char: u8)
; Blocks until ready to print, then prints r0.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
putc:
    ; Loop until the teletype is ready to accept another character.
    bicb    #TPS_READY_CMASK, @#TPS
    beq     putc

    movb    r0, @#TPB
    rts     pc  

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn putline(r0 start: char*, r1 end: char*)
; print (r0) through (r1) (exclusive).
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
putline:
    mov r2, -(sp)

    mov r1, r2
    mov r0, r1

1:
    cmp r1, r2
    beq 2f

    movb (r1)+, r0
    jsr  pc, putc
    br   1b

2:
    mov (sp)+, r2
    rts pc

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn read() -> r0 char: u8
; Blocks until char available; returns read char in r0.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
getc:
    bic #TKS_DONE_CMASK, @#TKS
    beq getc

    movb @#TKB, r0
    rts  pc


