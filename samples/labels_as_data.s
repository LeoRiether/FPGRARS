.data
vec: .word 1, 2, 3, 4, 5, 6, 7, 8, 9
vec_end:

fns: .word double, inc # labels in the data directive!

.macro print_char(%c)
    li a7 11
    li a0 %c
    ecall
.end_macro

.text
main:
    .macro load_vec()
        la a0 vec
        la a1 vec_end
    .end_macro

    load_vec()
    jal show

    la s0 fns

    load_vec()
    lw a2 0(s0) # double
    jal map

    load_vec()
    jal show

    load_vec()
    lw a2 4(s0) # inc
    jal map

    load_vec()
    jal show

main.exit:
	li a7 10
	ecall

# Prints an array
# a0 = begin
# a1 = end
show:
    mv a2 a0
    mv a3 a1

show.loop:
    bge a2 a3 show.exit

    li a7 1
    lw a0 0(a2)
    ecall

    print_char(32)

    addi a2 a2 4
    j show.loop

show.exit:
    print_char('\n')
    ret

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