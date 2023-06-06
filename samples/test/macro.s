.data
hw: .string "Hello World!\n"
linebreak_string: .ascii "\n"

.macro print_string(%label)
    li a7 4
    la a0 %label
    ecall
.end_macro

.macro linebreak
    print_string(linebreak_string)
.end_macro

.macro print_char(%c)
    li a7 11
    li a0 %c
    ecall
.end_macro

.macro print_string_lit(%str)
.data
var: .string %str

.text
    li a7 4
    la a0 var
    ecall
.end_macro

.macro print_repeat(%char, %n)
    sw t0 -4(sp)
    li t0 %n
    loop:
        print_char(%char)
        addi t0 t0 -1
        bnez t0 loop
    lw t0 -4(sp) 
    print_char('\n')
.end_macro

.text
    # https://github.com/LeoRiether/FPGRARS/issues/18
    print_string_lit("apple\n")
    print_string_lit("lemon\n")

    print_repeat('!', 10)
    print_repeat('!', 20)

    linebreak

    print_char('!')
    print_char('\n')

    li a7 10
    ecall
