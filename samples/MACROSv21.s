#########################################################################
# Definiçõe e Macros						v2.1	#
# Marcus Vinicius Lamar							#
# 2020/1								#
#########################################################################

######### Verifica se eh a DE1-SoC ###############
.macro DE1(%reg,%salto)
	li %reg, 0x10008000	# carrega tp
	bne gp, %reg, %salto	# Na DE1 gp = 0 ! Não tem segmento .extern
.end_macro

######### Verifica se tem ISA RV32IMF ###############
.macro TEM_F(%reg,%endereco)
	csrr %reg, misa
	andi %reg, %reg, 0x020
	bnez %reg, %endereco
.end_macro

######### Verifica se não tem ISA RV32IMF ###############
.macro NAOTEM_F(%reg,%endereco)
	csrr %reg, misa
	andi %reg, %reg, 0x020
	beqz %reg, %endereco
.end_macro

######### Verifica se tem ISA RV32IMF ou RV32IM ###############
.macro TEM_M(%reg,%endereco)
	csrr %reg, misa
	srli %reg, %reg, 12
	andi %reg, %reg, 0x001
	bnez %reg, %endereco
.end_macro

######### Verifica se não tem ISA RV32IMF ou RV32IM ###############
.macro NAOTEM_M(%reg,%endereco)
	csrr %reg, misa
	srli %reg, %reg, 12
	andi %reg, %reg, 0x001
	beqz %reg, %endereco
.end_macro

######### Macro para Multiplicação na ISA RV32I ######################
.macro MULTIPLY(%rd,%r1,%r2)
		addi 	sp, sp, -12
		sw	a0, 0(sp)
		sw	a1, 4(sp)
		sw	ra, 8(sp)
		
		mv 	a0, %r1
		mv 	a1, %r2
		jal 	__mulsi3
        	csrw	a0,uscratch
        	
		lw	a0, 0(sp)
		lw	a1, 4(sp)
		lw	ra, 8(sp)
		addi 	sp, sp, 12
		csrr	%rd,uscratch
.end_macro

######### Macro para Divisão unsigned por 10 na ISA RV32I ######################
### https://stackoverflow.com/questions/5558492/divide-by-10-using-bit-shifts
.macro DIVU10(%rd,%r1)
		addi 	sp, sp, -16
		sw	a0, 0(sp)
		sw	a4, 4(sp)
		sw	a5, 8(sp)
		sw	ra, 12(sp)
	mv 	a0, %r1
        srli    a4,a0,1
        srli    a5,a0,2
        add     a5,a4,a5
        srli    a4,a5,4
        add     a4,a4,a5
        srli    a5,a4,8
        add     a4,a5,a4
        srli    a5,a4,16
        add     a5,a5,a4
        srli    a5,a5,3
        slli    a4,a5,2
        add     a4,a4,a5
        slli    a4,a4,1
        sub     a0,a0,a4
        sltiu   a0,a0,10
        xori    a0,a0,1
        add     a0,a0,a5
        csrw	a0,uscratch 
        	lw	a0, 0(sp)
		lw	a4, 4(sp)
		lw	a5, 8(sp)
		lw	ra, 12(sp)
		addi 	sp, sp, 16	
	csrr	%rd,uscratch
.end_macro

######### Macro para Divisão por 10 na ISA RV32I ######################
.macro DIV10(%rd,%r1)
		addi 	sp,sp,-12
		sw	a0,0(sp)
		sw	a1,4(sp)
		sw	a2,8(sp)
		mv 	a2,%r1
		srai 	a1,a2,31
		mv 	a0,a2
		beqz 	a1,div10.pula1
		neg 	a0,a2
div10.pula1:	DIVU10(%rd,a0)
		beqz 	a1,div10.pula2
		neg 	%rd,%rd
div10.pula2:	csrw	%rd,uscratch
		lw	a0,0(sp)
		lw 	a1,4(sp)
		lw	a2,8(sp)
		addi 	sp,sp,12
		csrr	%rd,uscratch
.end_macro		


######### Macro para resto da divisão por 10 unsigned na ISA RV32I ######################
.macro REMU10(%rd,%r1)
		addi 	sp,sp,-16
		sw	a0,0(sp)
		sw	a1,4(sp)
		sw	a2,8(sp)
		sw	a3,12(sp)
		
		mv 	a3,%r1
		li 	a2,10
		DIVU10(a0,a3)
		MULTIPLY(a1,a0,a2)
		sub 	%rd,a3,a1		
				
		csrw	%rd,uscratch
		lw	a0,0(sp)
		lw 	a1,4(sp)
		lw	a2,8(sp)
		lw	a3,12(sp)
		addi 	sp,sp,16
		csrr	%rd,uscratch
.end_macro

######### Macro para resto da divisão por 10 na ISA RV32I ######################
.macro REM10(%rd,%r1)
		addi 	sp,sp,-16
		sw	a0,0(sp)
		sw	a1,4(sp)
		sw	a2,8(sp)
		sw	a3,12(sp)
		
		mv 	a3,%r1
		li 	a2,10
		DIV10(a0,a3)
		MULTIPLY(a1,a0,a2)
		sub 	%rd,a3,a1
		
		csrw	%rd,uscratch
		lw	a0,0(sp)
		lw 	a1,4(sp)
		lw	a2,8(sp)
		lw 	a3,12(sp)
		addi 	sp,sp,16
		csrr	%rd,uscratch
.end_macro	
			

#definicao do mapa de enderecamento de MMIO
.eqv VGAADDRESSINI0     0xFF000000
.eqv VGAADDRESSFIM0     0xFF012C00
.eqv VGAADDRESSINI1     0xFF100000
.eqv VGAADDRESSFIM1     0xFF112C00 
.eqv NUMLINHAS          240
.eqv NUMCOLUNAS         320
.eqv VGAFRAMESELECT	0xFF200604

.eqv KDMMIO_Ctrl	0xFF200000
.eqv KDMMIO_Data	0xFF200004

.eqv Buffer0Teclado     0xFF200100
.eqv Buffer1Teclado     0xFF200104

.eqv TecladoxMouse      0xFF200110
.eqv BufferMouse        0xFF200114

.eqv AudioBase		0xFF200160
.eqv AudioINL           0xFF200160
.eqv AudioINR           0xFF200164
.eqv AudioOUTL          0xFF200168
.eqv AudioOUTR          0xFF20016C
.eqv AudioCTRL1         0xFF200170
.eqv AudioCTRL2         0xFF200174

# Sintetizador - 2015/1
.eqv NoteData           0xFF200178
.eqv NoteClock          0xFF20017C
.eqv NoteMelody         0xFF200180
.eqv MusicTempo         0xFF200184
.eqv MusicAddress       0xFF200188


.eqv IrDA_CTRL 		0xFF20 0500	
.eqv IrDA_RX 		0xFF20 0504
.eqv IrDA_TX		0xFF20 0508

.eqv STOPWATCH		0xFF200510

.eqv LFSR		0xFF200514

.eqv KeyMap0		0xFF200520
.eqv KeyMap1		0xFF200524
.eqv KeyMap2		0xFF200528
.eqv KeyMap3		0xFF20052C

.eqv TimerLOW		0xFF200700
.eqv TimerHIGH		0xFF200704
.eqv InterLOW		0xFF200708
.eqv InterHIGH		0xFF20070C

.eqv FDIVIDER		0xFF200710


# Seta o uso do exception handler SYSTEM.s
.text
 	la 	tp, ExceptionHandling	# carrega em tp o endereço base das rotinas do sistema ECALL
 	csrw 	tp, utvec 		# seta utvec para o endereço tp
 	csrsi 	ustatus, 1 		# seta o bit de habilitação de interrupção em ustatus (reg 0)																																																				

