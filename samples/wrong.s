# ¯\_(ツ)_/¯

.data

something: .word 1, 2, 3, 4
something_else: .string "Something: ", "Something else: "

linebreak_str: .ascii "\n"

number: .word -1234

hello_str: .string "Hello World!"

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

.macro	mjal(%label)
	la tp, %label
	jalr ra, tp, 0
.end_macro

.text
	mjal(say_hello)

	lw a0 number

	li a7 10
	ecall

	say_hello:
		la a0 hello_str
		li a7 4
		ecall
		ret

