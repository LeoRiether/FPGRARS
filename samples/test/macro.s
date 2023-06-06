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

.text
    # This works: linebreak()
    linebreak

    print_char('!')
    print_char('\n')

    li a7 10
    ecall
