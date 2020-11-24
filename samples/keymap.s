###########################################
##                                       ##
##  Colors the screen if ESC is pressed  ##
##                                       ##
###########################################

.macro exit
    li a7 10
    ecall
.end_macro

.data

.text
main:
    li s0 0x0  # color
    li s1 1    # frame
    li s10 0   # last pressed key, only used for get_key

main.loop:
    mv a0 s0
    mv a1 s1
    jal print

    jal is_esc_pressed
    mv s0 zero
    beqz a0 not_red
    li s0 0x7
not_red:

    xori s1 s1 1
    j main.loop

main.exit:
    exit()

# a0 = color
# a1 = frame
print:
    slli a2 a1 20
    li t0 0xff000000
    or t0 t0 a2
    li t1 76800
    add t1 t1 t0

    slli a2 a0 8
    or a0 a0 a2
    slli a2 a0 16
    or a0 a0 a2

print.loop:
    bge t0 t1 print.exit

#     li t2 -250
# print.wait:
#     bgez t2 print.wait.out
#     addi t2 t2 1
#     j print.wait
# print.wait.out:

    sw a0 0(t0)

    addi t0 t0 4
    j print.loop

print.exit:
    li a2 0xff200604
    sb a1 0(a2)
    ret

# ESC should be scancode 1
is_esc_pressed:
    li a1 0xff200520
    lbu a0 0(a1)
    srli a0 a0 1
    andi a0 a0 1
    ret