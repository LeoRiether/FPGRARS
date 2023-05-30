##########################################
#                                        #
#        Busy sleeps for a second        #
#                                        #
##########################################

.data

.text
csrr s0 time

sleep:
    csrr s1 time
    sub s2 s1 s0

    li t0 1000
    bge s2 t0 sleep.exit
    j sleep

sleep.exit:
    li a7 1
    mv a0 s2
    ecall

    li a7 10
    ecall
