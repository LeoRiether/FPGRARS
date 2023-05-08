#######################################################
#                                                     #
#        Colors the screen red/blue many times        #
#                                                     #
#######################################################

.data
times: .word 1000
color: .byte 0xf8 0x07
.align 2

.text
main:
    # Sleep for some time for the window to show up in FPGRARS...
    li a0 300
    li a7 32
    ecall

    li s0 0 # s0 == color index
    lw s1 times # s1 = counter
    csrr s2 time # s2 = start time
main.loop:
    blez s1 exit
    
    la a0 color
    add a0 a0 s0
    lb a0 0(a0)
    jal draw

    addi s1 s1 -1
    xori s0 s0 1
    j main.loop

exit:
    csrr a0 time
    sub a0 a0 s2
    li a7 1
    ecall
    li a0 '\n'
    li a7 11
    ecall

    li a7 10
    ecall

# a0 = color
# frame is always zero
draw:
    li t0 0xff000000
    li t1 76800
    add t1 t1 t0

    # color as a word, could improve performance by like 4x
    # slli a2 a0 8
    # or a0 a0 a2
    # slli a2 a0 16
    # or a0 a0 a2

draw.loop:
    bge t0 t1 draw.exit

#     li t2 -250
# print.wait:
#     bgez t2 print.wait.out
#     addi t2 t2 1
#     j print.wait
# print.wait.out:

    sb a0 0(t0)

    addi t0 t0 1
    j draw.loop

draw.exit:
    # store frame index
    li a2 0xff200604
    sb zero 0(a2)
    ret
