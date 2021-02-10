.include "MACROSv21.s"

.data
raio: .word 110
lados: .word 5

tau: .float 6.284
dx: .float 0.02
to_radians: .float 0.01745329251

V: .space 404 # máximo de 50 vértices

.text

main:
	li s0 0 # frame
	li s1 0 # angulo

main.loop:
	li a0 0x00
	mv a1 s0
	li a7 48 # clear screen
	ecall

	# Check if the user wants to change the number of sides
	jal check_input

	# Circunferencia:
	la t0 raio
	lw a0 0(t0)
	li a1 0
	li a2 36 # polígono de 36 lados = circunferencia
	jal vertices
	li a1 0x07
	mv a2 s0
	jal desenha

	# Poligono:
	la t0 raio
	lw a0 0(t0) # raio
	mv a1 s1 # angulo
	la t0 lados
	lw a2 0(t0)
	jal vertices
	li a1 0xf8
	mv a2 s0
	jal desenha

	# Muda de frame
	li t0 0xFF200604
	sb s0 0(t0)
	xori s0 s0 1

	addi s1 s1 1
	li t0 360
	bge s1 t0 main.fix_angle

    jal stall

	j main.loop

main.fix_angle:
	sub s1 s1 t0
	j main.loop

main.exit:
	li a7 10
	ecall

# a0 = raio
# a1 = angulo
# a2 = numero de lados
vertices:
	addi sp sp -28
	sw ra 0(sp)
	sw s0 4(sp)
	sw s1 8(sp)
	fsw fs0 12(sp)
	fsw fs1 16(sp)
	fsw fs2 20(sp)
	fsw fs3 24(sp)

	fcvt.s.w fs0 a0 # fs0 = raio
	neg a1 a1
	fcvt.s.w fs1 a1 # fs1 = angulo
	la t0 to_radians
	flw ft0 0(t0)
	fmul.s fs1 fs1 ft0

	mv s0 a2 # s0 = numero de lados

	la t0 tau
	flw fs2 0(t0)
	fcvt.s.w ft0 s0
	fdiv.s fs2 fs2 ft0 # fs2 = incremento do angulo a cada passo

	la s1 V # output
	sw s0 0(s1) # salva o número de lados na primeira word de V
	addi s1 s1 4

vertices.loop:
	blez s0 vertices.exit

	fmv.s fa0 fs1
	li a0 1
	jal sin
	fmv.s fs3 fa0 # fs3 = sin(theta)

	fmv.s fa0 fs1
	li a0 0
	jal sin # cos

	# Multiplica pelo raio
	fmul.s fs3 fs3 fs0
	fmul.s fa0 fa0 fs0

	# Converte para inteiro (x = t0, y = t1)
	fcvt.w.s t0 fa0
	fcvt.w.s t1 fs3

	# Translada para o centro
	addi t0 t0 160
	addi t1 t1 120

	# Salva em V
	sw t0 0(s1)
	sw t1 4(s1)
	addi s1 s1 8

	fadd.s fs1 fs1 fs2
	addi s0 s0 -1
	j vertices.loop

vertices.exit:
	lw ra 0(sp)
	lw s0 4(sp)
	lw s1 8(sp)
	flw fs0 12(sp)
	flw fs1 16(sp)
	flw fs2 20(sp)
	flw fs3 24(sp)
	addi sp sp 28

	la a0 V
	ret

# a0 = int* V
# a1 = cor
# a2 = frame
desenha:
	mv t0 a0
	mv t6 a1
	mv t5 a2
	lw t1 0(t0) # t1 = numero de lados

	# Desenha V[n-1] -> V[0]
	slli a0 t1 3
	add a0 a0 t0
	lw a1 0(a0)
	lw a0 -4(a0)
	lw a2 4(t0)
	lw a3 8(t0)
	mv a4 t6
	mv a5 t5
	li a7 47
	ecall

	addi t1 t1 -1
	addi t0 t0 12

desenha.loop:
	blez t1 desenha.exit

	lw a0 -8(t0)
	lw a1 -4(t0)
	lw a2 0(t0)
	lw a3 4(t0)
	mv a4 t6
	mv a5 t5
	li a7 47
	ecall

	addi t0 t0 8
	addi t1 t1 -1
	j desenha.loop

desenha.exit:
	ret

# fa0 = theta
# a0 = 1 para sin(x), 0 para cos(x)
sin:
	fmv.s.x ft0 x0
	fadd.s fa1 fa0 ft0 # fa1 = theta
	fadd.s fa0 ft0 ft0 # fa0 = resposta
	fadd.s fa2 fa1 ft0 # fa2 = pow(theta, 2n + 1)

	li t0 1
	fcvt.s.w fa3 t0 # fa3 = (2n + 1)!

	bnez a0 sin.notcos # just sin(x)

sin.cos: # yup
	fmv.s fa2 fa3 # pow(theta, 2n=0) = 1

sin.notcos:

	mv t0 x0
sin.loop:
	li t1 10 # executa 11 vezes /shrug
	bgt t0 t1 sin.exit

	fdiv.s ft1 fa2 fa3

	andi t1 t0 1
	bnez t1 sin.sub # n é ímpar

sin.sum:
	fadd.s fa0 fa0 ft1
	j sin.control

sin.sub:
	fsub.s fa0 fa0 ft1

sin.control:

	# atualiza pow(theta, 2n+1)
	fmul.s fa2 fa2 fa1
	fmul.s fa2 fa2 fa1

	beqz a0 sin.cos.control # cos(x) control

	# atualiza (2n + 1)!
	addi t0 t0 1

	# vezes 2, 4, 6, ...
	slli t1 t0 1
	fcvt.s.w ft1 t1
	fmul.s fa3 fa3 ft1

	# vezes 3, 5, 7, ...
	addi t1 t1 1
	fcvt.s.w ft1 t1
	fmul.s fa3 fa3 ft1

	j sin.loop

sin.cos.control:
	# atualiza (2n)!
	# vezes 1, 3, 5, ...
	slli t1 t0 1
	addi t1 t1 1
	fcvt.s.w ft1 t1
	fmul.s fa3 fa3 ft1

	# vezes 2, 4, 6, ...
	addi t1 t1 1
	fcvt.s.w ft1 t1
	fmul.s fa3 fa3 ft1

	addi t0 t0 1
	j sin.loop

sin.exit:
	ret

check_input:
	li t0 0xff200000
	lb a0 0(t0)
	andi a0 a0 1
	beqz a0 check_input.exit

	lb a0 4(t0)
	li a1 '0'
	sub a0 a0 a1

	# Make sure the number is at least 2
	li a1 2
	blt a0 a1 check_input.exit

	la t0 lados
	sw a0 0(t0)

check_input.exit:
	ret


# busy sleep for 8ms
stall:
	li t0 8
	csrr a0 time
stall.loop:
	csrr a1 time
	sub a1 a1 a0

	bge a1 t0 stall.exit

	j stall.loop

stall.exit:
	ret

.include "SYSTEMv21.s"