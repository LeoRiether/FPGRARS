.macro TWO(%f %g)
    %f
    %g
.end_macro

.macro EXIT(%1)
    li a7 %1
    ecall
.end_macro

.macro NOPE
    nop
.end_macro

TWO(EXIT(10), NOPE)