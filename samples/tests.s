.data
    number: 0x1234

.macro print_hex(%reg)
    mv a0 %reg
    li a7 34
    ecall
    li a0 '\n'
    li a7 11
    ecall
.end_macro

.macro sbrk(%bytes)
    li a7 9
    li a0 %bytes
    ecall
.end_macro

.macro sbrk_print(%bytes)
    sbrk( %bytes)
    print_hex(a0 )
.end_macro

.text
    sbrk_print( 1 )
    sbrk_print( 2)
    sbrk_print(4 )
    sbrk_print(16)
    sbrk_print(1)
    sbrk_print(0)

    sbrk(4)
    mv s0 a0
    print_hex(s0)

    lw a0 0(s0)
    print_hex(a0) # initially, 0x0 is stored in 0(s0)

    lw a0 number
    sw a0 0(s0)

    lw a0 0(s0)
    print_hex(a0) # now, 0x1234 should be stored in 0(s0)

