.include "../../RARS/utf8_MACROSv21.s"

.data

.text
main:
    li s0 320 # posição
    li s1 0 # frame

main.loop:
    li s10 1000
stall:
    li a7 48
    li a0 0
    mv a1 s1
    ecall

    li a7 47
    mv a0 s0
    li a1 0
    li a2 319
    sub a2 a2 s0
    li a3 239
    li a4 0x1f
    mv a5 s1
    ecall

    addi s10 s10 -1
    bgez s10 stall

    li t0 VGAFRAMESELECT
    sw s1 (t0)
    xori s1 s1 1

    addi s0 s0 -1
    bltz s0 exit
    j main.loop

main.fix:
    addi s0 s0 320
    j main.loop

exit:
    li a7 10
    ecall

.include "../../RARS/utf8_SYSTEMv21.s"