.data
str1: .asciz "Hello World!\n"

# Suppose I forgot to add the .text directive!
li t0, 10
loop:
    addi t0, t0, -1
    bne t0, zero, loop
