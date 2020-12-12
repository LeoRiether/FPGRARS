.data

    label: .word 123

.text

    la a0 label
    sw a1 0(a0)
    sw a1 (a0)
    sw a1 label a0
