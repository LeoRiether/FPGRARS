.global main

.eqv WRITE_INT 1
.eqv WRITE_CHAR 11
.eqv EXIT 10

.macro dump_registers()
    addi sp sp -4
    sw ra 0(sp)
    jal ra, dump_registers.fn
    lw ra 0(sp)
    addi sp sp 4
.end_macro

.text
main:
    .include "randomly_generated_instructions.s"

exit:
    li a7 EXIT
    li a0 0
    ecall

dump_registers.fn:
    # Save registers
    # skip x0-x4 (zero, ra, sp, gp, tp) 
    addi sp sp -104
    sw x5 0(sp)
    sw x6 4(sp)
    sw x7 8(sp)
    sw x8 12(sp)
    sw x9 16(sp)
    sw x10 20(sp)
    sw x11 24(sp)
    sw x12 28(sp)
    sw x13 32(sp)
    sw x14 36(sp)
    sw x15 40(sp)
    sw x16 44(sp)
    sw x17 48(sp)
    sw x18 52(sp)
    sw x19 56(sp)
    sw x20 60(sp)
    sw x21 64(sp)
    sw x22 68(sp)
    sw x23 72(sp)
    sw x24 76(sp)
    sw x25 80(sp)
    sw x26 84(sp)
    sw x27 88(sp)
    sw x28 92(sp)
    sw x29 96(sp)
    sw x30 100(sp)
    sw x31 104(sp)

    mv t0 sp
    addi t1 sp 104 
dump_registers.loop:
    li a7 WRITE_INT
    lw a0 0(t0)
    ecall
    li a7 WRITE_CHAR
    li a0 ' '
    ecall

    addi t0 t0 4
    bge t1 t0 dump_registers.loop

    li a7 WRITE_CHAR
    li a0 '\n'
    ecall

dump_registers.exit:
    # Restore registers
    lw x5 0(sp)
    lw x6 4(sp)
    lw x7 8(sp)
    lw x8 12(sp)
    lw x9 16(sp)
    lw x10 20(sp)
    lw x11 24(sp)
    lw x12 28(sp)
    lw x13 32(sp)
    lw x14 36(sp)
    lw x15 40(sp)
    lw x16 44(sp)
    lw x17 48(sp)
    lw x18 52(sp)
    lw x19 56(sp)
    lw x20 60(sp)
    lw x21 64(sp)
    lw x22 68(sp)
    lw x23 72(sp)
    lw x24 76(sp)
    lw x25 80(sp)
    lw x26 84(sp)
    lw x27 88(sp)
    lw x28 92(sp)
    lw x29 96(sp)
    lw x30 100(sp)
    lw x31 104(sp)
    addi sp sp 104

    ret
