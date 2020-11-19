# ¯\_(ツ)_/¯

.data
	.word 1, 2, 3, 4
fns:.word double, inc # labels!

.text


main.exit:
	li a7 10
	ecall

# a0 = begin
# a1 = end
# a2 = map function (Fn(u32): u32)
map:
	addi sp sp -16
	sw s0 0(sp)
	sw s1 4(sp)
	sw s2 8(sp)
	sw ra 12(sp)

	mv s0 a0
	mv s1 a1
	mv s2 a2

map.loop:
	bge s0 s1 map.exit

	lw a0 0(s0)
	jalr ra s2 0
	sw a0 0(s0)

	addi s0 s0 4
	j map.loop

map.exit:

	lw s0 0(sp)
	lw s1 4(sp)
	lw s2 8(sp)
	lw ra 12(sp)
	addi sp sp 12
	ret

# a0 = x
# returns 2x
double:
	slli a0 a0 1
	ret

# a0 = x
# return x + 1
inc:
	addi a0 a0 1
	ret