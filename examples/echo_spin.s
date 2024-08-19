
    STACK_TOP = 150000 

    TPS = 177564
    TPB = TPS + 2
    TPS_READY_CMASK = 177
    TKS = 177560
    TKB = TKS + 2
    TKS_DONE_CMASK = 177577

    . = 400

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn _start()
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
_start:
    mov #STACK_TOP, sp
    mov #buf, r1

read_loop:
    jsr     pc, read

    ; If we're done with the line, echo it.
    cmpb    #'\n, r0
    beq     do_print

    ; If we hit the line length limit, drop characters other than \n.
    cmpb    r1, #buf_end
    beq     read_loop

    ; If we have room, save character in buffer, echo it, then and read another.
    movb    r0, (r1)+
    jsr     pc, print
    br      read_loop


do_print:
    ; Echo the newline terminating the original line.
    movb    #'\n, r0
    jsr     pc, print

    ; Save the terminating newline to the buffer and print.
    movb    #'\n, (r1)+
    mov     #buf, r0
    jsr     pc, print_line

    mov     #buf, r1
    br      read_loop


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn print_line(r0 start: char*, r1 end: char*)
; print (r0) through (r1) (exclusive).
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
print_line:
    mov r2, -(sp)

    mov r1, r2
    mov r0, r1

print_loop:
    cmp r1, r2
    beq print_loop_done

    movb (r1)+, r0
    jsr  pc, print
    br   print_loop

print_loop_done:
    mov (sp)+, r2
    rts pc


;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn read() -> r0 char: u8
; Blocks until char available; returns read char in r0.
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
read:
    bic #TKS_DONE_CMASK, @#TKS
    beq read

    movb @#TKB, r0
    rts  pc


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

BUFLEN = 72.
buf:
    . = . + BUFLEN
buf_end:
    .word 0 ; extra space for a newline

