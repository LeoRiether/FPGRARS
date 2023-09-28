.data
hw: .string "Hello World!\n"
linebreak_string: .asciz "\n"

plus: " + "
equals: " = "

nums: .word 0x01, 2

.macro print_string(%label)
    li a7 4
    la a0 %label
    ecall
.end_macro

.macro linebreak
    print_string(linebreak_string)
.end_macro

.macro print_int
    li a7 1
    ecall
.end_macro

.text
    print_string(hw)

    la s0 nums

    lw a0 0(s0)
    print_int()

    print_string(plus)

    lw a0 4(s0)
    print_int()

    print_string(equals)

    lw a0 0(s0)
    lw a1 4(s0)
    add a0 a0 a1
    print_int()

    linebreak

    li a7 10
    ecall
