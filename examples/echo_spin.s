    STACK_TOP = 150000 

    TPS = 177564
    TPB = TPS + 2
    TPS_READY_CMASK = 177
    TKS = 177560
    TKB = TKS + 2
    TKS_DONE_CMASK = 177577

    . = 400

_start:
    mov #STACK_TOP, sp
    mov #buf, r1

read_loop:
    jsr     pc, read
    jsr     pc, print
    movb    r0, (r1)+

    cmpb    #'\n, r0
    beq     do_print

    cmpb    r1, #buf_end
    beq     out_of_room

    br      read_loop

out_of_room:
    mov     #'\n, r0
    jsr     pc, print

do_print:
    mov #buf, r0
    jsr pc, print_line

    mov #buf, r1
    br  read_loop



; print (r0) through (r1) (exclusive)
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


; Blocks until char available; returns read char in r0
read:
    bic #TKS_DONE_CMASK, @#TKS
    beq read

    movb @#TKB, r0
    rts  pc


; Blocks until ready to print; char passed r0
print:
    ; Loop until the teletype is ready to accept another character.
    bicb #TPS_READY_CMASK, @#TPS
    beq  print

    movb r0, @#TPB
    rts  pc  

BUFLEN = 110
buf:
    . = . + BUFLEN
buf_end:
    .word 0 ; extra space for a newline

