.data
    LABEL: .string "Hello World!!!\n"

.text

.macro load_half(%register,%var_address)
    li %register,4 # Comments here
    la a0,%var_address # Comments there
    ecall
.end_macro

    load_half(a7,LABEL )

