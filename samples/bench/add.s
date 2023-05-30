.data
number: .word 20000000
bignumber: .word 200000000

.text
main:
    # lw s0 number
    la s0 number
    lw s0 0(s0)
    csrr s1 time

loop:
    add t0 t1 t2
    add t0 t1 t2
    add t0 t1 t2
    add t0 t1 t2
    add t0 t1 t2

    addi s0 s0 -1
    bnez s0 loop

exit:
    csrr s2 time
    sub a0 s2 s1
    li a7 1
    ecall
    li a7 11
    li a0 '\n'
    ecall

    li a7 10
    li a0 0
    ecall

