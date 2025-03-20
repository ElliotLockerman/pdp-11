
    STACK_TOP = 150000 

    . = 400

;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
; fn _start()
;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;
_start:
    mov #STACK_TOP, sp
    mov #buf, r1

    ; Main loop.
1:
    jsr     pc, getc

    ; If we're done with the line, print it.
    cmpb    #'\n, r0
    beq     2f

    ; If we hit the line length limit, drop characters other than \n and loop.
    cmpb    r1, #buf_end
    beq     1b

    ; If we have room, save character in buffer, echo it and loop.
    movb    r0, (r1)+
    jsr     pc, putc
    br      1b


    ; Print the buffer, then loop.
2:
    ; Echo the newline terminating the original line.
    movb    #'\n, r0
    jsr     pc, putc

    ; Save the terminating newline to the buffer and print the line.
    movb    #'\n, (r1)+
    mov     #buf, r0
    jsr     pc, putline

    mov     #buf, r1
    br      1b

BUFLEN = 72.
buf:
    . = . + BUFLEN
buf_end:
    .word 0 ; extra space for a newline

