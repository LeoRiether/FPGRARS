
###########################
##                       ##
##  Also waits a second  ##
##                       ##
###########################

.data

.text
    csrr s0 time

    # Sleep ecall
    li a7 32
    li a0 1000
    ecall

    csrr s1 time

    li a7 1
    sub a0 s1 s0
    ecall

    li a7 10
    ecall