.include "../../RARS/utf8_MACROSv21.s"

.data

.text
    li a7 47
    li a0 0
    li a1 0
    li a2 100
    li a3 100
    li a4 0x3f
    li a5 0
    ecall

stall: j stall

.include "../../RARS/utf8_SYSTEMv21.s"