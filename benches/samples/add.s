.eqv N 20000000

.text
main:
    li s0 N

loop:
    add t0 t1 t2
    add t0 t1 t2
    add t0 t1 t2
    add t0 t1 t2
    add t0 t1 t2

    addi s0 s0 -1
    bnez s0 loop
