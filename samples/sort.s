##########################################################
##                                                      ##
##  Reads an array of 10 integers and prints it sorted  ##
##                                                      ##
##########################################################

.data

.text
    # Allocates the array in the stack
	# [sp .. s1 .. s0)
	mv s0 sp
	addi sp sp -40
	mv s1 sp
control:
	bge s1 s0 print

    li a7 5
    ecall
	sw a0 0(s1)

	mv a0 s1
	mv a1 sp
	jal ra fix

	addi s1 s1 4
	j control

print:
	mv s1 sp
pcontrol:
	bge s1 s0 exit
	lw a0 0(s1)
	li a7 1
	ecall
	li a0 ' '
	li a7 11
	ecall

	addi s1 s1 4
	j pcontrol

exit:

    li a7 11
    li a0 '\n'
    ecall
	li a7 10
    li a0 0
	ecall

# Moves the last element of the array to the front until it gets to the right position
# fix(int* last, int* first)
fix:
	ble a0 a1 fexit

	lw t0 0(a0)
	lw t1 -4(a0)
	bge t0 t1 fix_no_swap

	# swap
	sw t1 0(a0)
	sw t0 -4(a0)

fix_no_swap:
	addi a0 a0 -4
	j fix

fexit:
	ret
