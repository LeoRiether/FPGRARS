# ¯\_(ツ)_/¯

.data

something: .word 1, 2, 3, 4
something_else: .string "Something: ", "Something else: "

.text
	li a0 0
	li a1 1234
	li a7 40
	ecall

	li s0 10
loop:
	blez s0 exit
	li a7 42
	li a0 0
	li a1 2
	ecall

	li a7 1
	ecall

	addi s0 s0 -1
	j loop
exit:
	li a7 10
	ecall
