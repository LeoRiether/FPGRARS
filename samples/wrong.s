# ¯\_(ツ)_/¯

.data

something: .word 1, 2, 3, 4
something_else: .string "Something: ", "Something else: "

linebreak_str: .ascii "\n"

.macro linebreak
	li a7 4
	la a0 linebreak_str
	ecall
.end_macro

.macro print_int
	li a7 1
	ecall
	linebreak()
.end_macro

.macro print_int(%reg)
	mv a0 %reg
	print_int
.end_macro

.text
	li t0 1
	slli t0 t0 1025
	print_int(t0)