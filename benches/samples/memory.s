.data
N: .word 10000000
array: .space 32

.text
    lw s0, N
    la t0, array
loop:
    lw t1, 0(t0)
    lw t1, 0(t0)
    lw t1, 0(t0)
    lw t1, 0(t0)
    lw t1, 0(t0)

    addi s0, s0, -1
    bgez s0, loop

