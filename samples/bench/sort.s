#############################################################
##                                                         ##
##  Fills an array with the worst case for insertion sort  ##
##  and prints it sorted                                   ##
##                                                         ##
#############################################################

.data
N: .word 10000

.text
    # Allocates the array in the stack with N integers
    # [sp .. s0)
    mv s0 sp
    lw t0 N
    slli t0 t0 2
    sub sp sp t0
    mv s1 sp

    # Fill with the worst case (array sorted in reverse)
    mv a0 sp
    mv a1 s0
    jal fill

    # Print whether it's sorted or not (should print 0)
    mv a0 sp
    mv a1 s0
    jal check_sorted
    li a7 1
    ecall

    # Save time
    # csrr s7 time

    # Sort
    mv a0 sp
    mv a1 s0
    jal sort

    # Save time 
    # csrr t0 time
    # sub s7 t0 s7 # s7 == time elapsed

    # Print whether it's sorted or not again (should print 1)
    mv a0 sp
    mv a1 s0
    jal check_sorted
    li a7 1
    ecall

    # Print time elapsed
    # li a0 '\n'
    # li a7 11
    # ecall
    # mv a0 s7
    # li a7 1 
    # ecall
    # li a0 '\n'
    # li a7 11
    # ecall

exit:
    li a7 10
    ecall

# a0 = begin
# a1 = end
sort:
    addi sp sp -12
    sw ra 0(sp)
    sw s0 4(sp)
    sw s1 8(sp)

    mv s0 a0
    mv s1 a1

sort.loop:
    bge s0 s1 sort.exit

    mv a0 s0
    mv a1 s1
    jal fix

    addi s1 s1 -4
    j sort.loop

sort.exit:
    lw ra 0(sp)
    lw s0 4(sp)
    lw s1 8(sp)
    addi sp sp 12
    ret

# Moves the last element of the array to the front until it gets to the right position
# fix(int* last, int* first)
fix:
    bge a0 a1 fexit

    lw t0 0(a1)
    lw t1 -4(a1)
    bge t0 t1 fix_no_swap

    # swap
    sw t1 0(a1)
    sw t0 -4(a1)

fix_no_swap:
    addi a1 a1 -4
    j fix

fexit:
    ret

# a0 = begin
# a1 = end
fill:
    sub t0 a1 a0 # t0 = first index
fill.loop:
    bge a0 a1 fill.exit

    sw t0 0(a0)

    addi a0 a0 4
    addi t0 t0 -1
    j fill.loop

fill.exit:
    ret

# a0 = begin
# a1 = end
# returns in a0 = 1 iff the array is sorted
check_sorted:
    addi a1 a1 -4
cs_loop:
    bge a0 a1 cs_is_sorted

    lw t0 0(a0)
    lw t1 4(a0)
    bgt t0 t1 cs_not_sorted

    j check_sorted

cs_is_sorted:
    li a0 1
    ret
cs_not_sorted:
    li a0 0
    ret
