#########################################################################
# Rotina de tratamento de excecao e interrupcao		v2.1		#
# Lembre-se: Os ecalls originais do Rars possuem precedencia sobre	#
# 	     estes definidos aqui					#
# Os ecalls 1XX usam o BitMap Display e Keyboard Display MMIO Tools	#
# Usar o RARS14_Custom4	(misa)						#
# Marcus Vinicius Lamar							#
# 2020/1								#
#########################################################################

# incluir o MACROSv20.s no início do seu programa!!!!

.data
.align 2

# Tabela de caracteres desenhados segundo a fonte 8x8 pixels do ZX-Spectrum
LabelTabChar:
.word 	0x00000000, 0x00000000, 0x10101010, 0x00100010, 0x00002828, 0x00000000, 0x28FE2828, 0x002828FE, 
	0x38503C10, 0x00107814, 0x10686400, 0x00004C2C, 0x28102818, 0x003A4446, 0x00001010, 0x00000000, 
	0x20201008, 0x00081020, 0x08081020, 0x00201008, 0x38549210, 0x00109254, 0xFE101010, 0x00101010, 
	0x00000000, 0x10081818, 0xFE000000, 0x00000000, 0x00000000, 0x18180000, 0x10080402, 0x00804020, 
	0x54444438, 0x00384444, 0x10103010, 0x00381010, 0x08044438, 0x007C2010, 0x18044438, 0x00384404, 
	0x7C482818, 0x001C0808, 0x7840407C, 0x00384404, 0x78404438, 0x00384444, 0x1008047C, 0x00202020, 
	0x38444438, 0x00384444, 0x3C444438, 0x00384404, 0x00181800, 0x00001818, 0x00181800, 0x10081818, 
	0x20100804, 0x00040810, 0x00FE0000, 0x000000FE, 0x04081020, 0x00201008, 0x08044438, 0x00100010, 
	0x545C4438, 0x0038405C, 0x7C444438, 0x00444444, 0x78444478, 0x00784444, 0x40404438, 0x00384440,
	0x44444478, 0x00784444, 0x7840407C, 0x007C4040, 0x7C40407C, 0x00404040, 0x5C404438, 0x00384444, 
	0x7C444444, 0x00444444, 0x10101038, 0x00381010, 0x0808081C, 0x00304848, 0x70484444, 0x00444448, 
	0x20202020, 0x003C2020, 0x92AAC682, 0x00828282, 0x54546444, 0x0044444C, 0x44444438, 0x00384444, 
	0x38242438, 0x00202020, 0x44444438, 0x0C384444, 0x78444478, 0x00444850, 0x38404438, 0x00384404, 
	0x1010107C, 0x00101010, 0x44444444, 0x00384444, 0x28444444, 0x00101028, 0x54828282, 0x00282854, 
	0x10284444, 0x00444428, 0x10284444, 0x00101010, 0x1008047C, 0x007C4020, 0x20202038, 0x00382020, 
	0x10204080, 0x00020408, 0x08080838, 0x00380808, 0x00442810, 0x00000000, 0x00000000, 0xFE000000, 
	0x00000810, 0x00000000, 0x3C043800, 0x003A4444, 0x24382020, 0x00582424, 0x201C0000, 0x001C2020, 
	0x48380808, 0x00344848, 0x44380000, 0x0038407C, 0x70202418, 0x00202020, 0x443A0000, 0x38043C44, 
	0x64584040, 0x00444444, 0x10001000, 0x00101010, 0x10001000, 0x60101010, 0x28242020, 0x00242830, 
	0x08080818, 0x00080808, 0x49B60000, 0x00414149, 0x24580000, 0x00242424, 0x44380000, 0x00384444, 
	0x24580000, 0x20203824, 0x48340000, 0x08083848, 0x302C0000, 0x00202020, 0x201C0000, 0x00380418, 
	0x10381000, 0x00101010, 0x48480000, 0x00344848, 0x44440000, 0x00102844, 0x82820000, 0x0044AA92, 
	0x28440000, 0x00442810, 0x24240000, 0x38041C24, 0x043C0000, 0x003C1008, 0x2010100C, 0x000C1010, 
	0x10101010, 0x00101010, 0x04080830, 0x00300808, 0x92600000, 0x0000000C, 0x243C1818, 0xA55A7E3C, 
	0x99FF5A81, 0x99663CFF, 0x10280000, 0x00000028, 0x10081020, 0x00081020

# scancode -> ascii
LabelScanCode:
#  	0     1     2     3     4     5     6     7     8     9     A     B     C     D     E     F
.byte 	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, # 00 a 0F
	0x00, 0x00, 0x00, 0x00, 0x00, 0x71, 0x31, 0x00, 0x00, 0x00, 0x7a, 0x73, 0x61, 0x77, 0x32, 0x00, # 10 a 1F 
	0x00, 0x63, 0x78, 0x64, 0x65, 0x34, 0x33, 0x00, 0x00, 0x20, 0x76, 0x66, 0x74, 0x72, 0x35, 0x00, # 20 a 2F   29 espaco => 20
	0x00, 0x6e, 0x62, 0x68, 0x67, 0x79, 0x36, 0x00, 0x00, 0x00, 0x6d, 0x6a, 0x75, 0x37, 0x38, 0x00, # 30 a 3F 
	0x00, 0x2c, 0x6b, 0x69, 0x6f, 0x30, 0x39, 0x00, 0x00, 0x2e, 0x2f, 0x6c, 0x3b, 0x70, 0x2d, 0x00, # 40 a 4F 
	0x00, 0x00, 0x27, 0x00, 0x00, 0x3d, 0x00, 0x00, 0x00, 0x00, 0x0A, 0x5b, 0x00, 0x5d, 0x00, 0x00, # 50 a 5F   5A Enter => 0A (= ao Rars)
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x31, 0x00, 0x34, 0x37, 0x00, 0x00, 0x00, # 60 a 6F   66 Backspace => 08
	0x30, 0x2e, 0x32, 0x35, 0x36, 0x38, 0x00, 0x00,	0x00, 0x2b, 0x33, 0x2d, 0x2a, 0x39, 0x00, 0x00, # 70 a 7F 
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00						   		# 80 a 85
# scancode -> ascii (com shift)
LabelScanCodeShift:
.byte   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
	0x00, 0x00, 0x00, 0x00, 0x00, 0x51, 0x21, 0x00, 0x00, 0x00, 0x5a, 0x53, 0x41, 0x57, 0x40, 0x00, 
	0x00, 0x43, 0x58, 0x44, 0x45, 0x24, 0x23, 0x00, 0x00, 0x00, 0x56, 0x46, 0x54, 0x52, 0x25, 0x00, 
	0x00, 0x4e, 0x42, 0x48, 0x47, 0x59, 0x5e, 0x00, 0x00, 0x00, 0x4d, 0x4a, 0x55, 0x26, 0x2a, 0x00, 
	0x00, 0x3c, 0x4b, 0x49, 0x4f, 0x29, 0x28, 0x00, 0x00, 0x3e, 0x3f, 0x4c, 0x3a, 0x50, 0x5f, 0x00, 
	0x00, 0x00, 0x22, 0x00, 0x00, 0x2b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7b, 0x00, 0x7d, 0x00, 0x00, 
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 
	0x00, 0x00, 0x00, 0x00, 0x00, 0x00

.align 2

#buffer do ReadString, ReadFloat, SDread, etc. 512 caracteres/bytes
TempBuffer:
.space 512

# tabela de conversao hexa para ascii
TabelaHexASCII:		.string "0123456789ABCDEF  "
NumDesnormP:		.string "+desnorm"
NumDesnormN:		.string "-desnorm"
NumZero:		.string "0.00000000"
NumInfP:		.string "+Infinity"
NumInfN:		.string "-Infinity"
NumNaN:			.string "NaN"

# tabela de causa de exceções 
Cause0: 		.string "Error: 0 Instruction address misaligned "
Cause1: 		.string "Error: 1 Instruction access fault "
Cause2: 		.string "Error: 2 Ilegal Instruction "
Cause4: 		.string "Error: 4 Load address misaligned "
Cause5:			.string "Error: 5 Load access fault "
Cause6: 		.string "Error: 6 Store address misaligned "
Cause7: 		.string "Error: 7 Store access fault "
CauseD:			.string "Error: Unknown "
CauseE:			.string "Error: Ecall "

PC:     		.string "PC: "
Addrs:  		.string "Addrs: "
Instr:  		.string "Instr: "


### Obs.: a forma 'LABEL: instrucao' embora fique feio facilita o debug no Rars, por favor nao reformatar!!!

########################################################################################
.text


###### Devem ser colocadas aqui as identificações das interrupções e exceções ###

	csrwi ucause,1		# caso ocorra dropdown vai gerar exceção de instrução inválida

ExceptionHandling:	addi 	sp, sp, -8 	# salva 2 registradores utilizados para comparar ucause
	sw 	t0, 0(sp)
	sw 	s10, 4(sp)

	csrr	s10,ucause     # le o ucause e salva em s10
	
	li 	t0, 8
	bne 	t0, s10, errorExceptions  	# Não é ecall - nem precisa arrumar a pilha!

	lw 	t0, 0(sp)			# É ecall
    	lw 	s10, 4(sp)  			# recupera registradores usados
    	addi 	sp, sp, 8			
	j 	ecallException
	
######################################################
###############   Exceções de Erros   ################
######################################################

errorExceptions: csrr 	s11, utval      # le o utval da exceção e salva em s11	
	addi 	a0, zero, 0xc0 		## printa tela de azul
	addi 	a1, zero, 0
	addi 	a7, zero, 148
	jal 	clsCLS
	
#  Instruction address misaligned 	
End_Cause0:	li 	t0, 0
		bne 	t0, s10, End_Cause1
		la 	a0, Cause0
		li 	a1, 0
		li 	a2, 1
		li 	a3, 0x0000c0ff
		jal	printString
		j	End_uepc
	
# Instruction access fault 
End_Cause1:	li 	t0, 1
		bne 	t0, s10, End_Cause2
		la 	a0, Cause1
		li 	a1, 0
		li 	a2, 1
		li 	a3, 0x0000c0ff
		jal 	printString	
		j	End_uepc
			
# Ilegal Instruction 
End_Cause2:	li 	t0, 2
		bne 	t0, s10, End_Cause4
		la 	a0, Cause2
		li 	a1, 0
		li 	a2, 1
		li 	a3, 0x0000c0ff
		jal 	printString
		
		la 	a0, Instr
		j	End_utval
	
# Load address misaligned 
End_Cause4:	addi 	t0, zero, 4
		bne	t0, s10, End_Cause5
		la 	a0, Cause4
		li 	a1, 0
		li 	a2, 1
		li 	a3, 0x0000c0ff
		jal	printString
		
		la 	a0, Addrs
		j	End_utval
		
# Load access fault 
End_Cause5:	li 	t0, 5
		bne 	t0, s10, End_Cause6
		la 	a0, Cause5
		li 	a1, 0
		li 	a2, 1
		li 	a3, 0x0000c0ff
		jal 	printString	
		
		la 	a0, Addrs
		j	End_utval
		
#Store address misaligned 
End_Cause6:	li 	t0, 6
		bne 	t0, s10, End_Cause7
		la 	a0, Cause6
		li 	a1, 0
		li 	a2, 1
		li 	a3, 0x0000c0ff
		jal 	printString
		
		la 	a0, Addrs
		j	End_utval
	
# Store access fault 
End_Cause7:	li 	t0, 7
		bne 	t0, s10, End_CauseD
		la 	a0, Cause7
		li 	a1, 0
		li 	a2, 1
		li 	a3, 0x0000c0ff
		jal 	printString
		
		la 	a0, Addrs
		j	End_utval

# Exception Unknown
End_CauseD: 	la 	a0, CauseD
		li 	a1, 0
		li 	a2, 1
		li 	a3, 0x0000c0ff
		jal 	printString
		
		la 	a0, Addrs


End_utval:	li 	a1, 0
		li 	a2, 24
		li 	a3, 0x000c0ff
		jal	printString
		
		mv 	a0, s11
		li 	a1, 56
		li 	a2, 24
		li 	a3, 0x0000c0ff
		jal 	printHex
	

End_uepc: 	la 	a0, PC 		# Imprime o pc em que a exceção ocorreu
		li 	a1, 0
		li 	a2, 12
		li 	a3, 0x000c0ff
		jal 	printString
		
		csrr 	a0, uepc	# Le uepc	
		li	a1, 28
		li 	a2, 12
		li 	a3, 0x0000c0ff
		jal 	printHex	
		
		j goToExit 		# encerra execução



######################################################
#############   exceção de ECALL   ###################
######################################################
ecallException:   addi    sp, sp, -264              # Salva todos os registradores na pilha
    sw     x1,    0(sp)
    sw     x2,    4(sp)
    sw     x3,    8(sp)
    sw     x4,   12(sp)
    sw     x5,   16(sp)
    sw     x6,   20(sp)
    sw     x7,   24(sp)
    sw     x8,   28(sp)
    sw     x9,   32(sp)
    sw     x10,  36(sp)
    sw     x11,  40(sp)
    sw     x12,  44(sp)
    sw     x13,  48(sp)
    sw     x14,  52(sp)
    sw     x15,  56(sp)
    sw     x16,  60(sp)
    sw     x17,  64(sp)
    sw     x18,  68(sp)
    sw     x19,  72(sp)
    sw     x20,  76(sp)
    sw     x21,  80(sp)
    sw     x22,  84(sp)
    sw     x23,  88(sp)
    sw     x24,  92(sp)
    sw     x25,  96(sp)
    sw     x26, 100(sp)
    sw     x27, 104(sp)
    sw     x28, 108(sp)
    sw     x29, 112(sp)
    sw     x30, 116(sp)
    sw     x31, 120(sp)
    NAOTEM_F(s8,ecallException.pula)
    fsw    f0,  124(sp)
    fsw    f1,  128(sp)
    fsw    f2,  132(sp)
    fsw    f3,  136(sp)
    fsw    f4,  140(sp)
    fsw    f5,  144(sp)
    fsw    f6,  148(sp)
    fsw    f7,  152(sp)
    fsw    f8,  156(sp)
    fsw    f9,  160(sp)
    fsw    f10, 164(sp)
    fsw    f11, 168(sp)
    fsw    f12, 172(sp)
    fsw    f13, 176(sp)
    fsw    f14, 180(sp)
    fsw    f15, 184(sp)
    fsw    f16, 188(sp)
    fsw    f17, 192(sp)
    fsw    f18, 196(sp)
    fsw    f19, 200(sp)
    fsw    f20, 204(sp)
    fsw    f21, 208(sp)
    fsw    f22, 212(sp)
    fsw    f23, 216(sp)
    fsw    f24, 220(sp)
    fsw    f25, 224(sp)
    fsw    f26, 228(sp)
    fsw    f27, 232(sp)
    fsw    f28, 236(sp)
    fsw    f29, 240(sp)
    fsw    f30, 244(sp)
    fsw    f31, 248(sp)
 ecallException.pula:   
    # Zera os valores dos registradores temporarios
    add     t0, zero, zero
    add     t1, zero, zero
    add     t2, zero, zero
    add     t3, zero, zero
    add     t4, zero, zero
    add     t5, zero, zero
    add     t6, zero, zero



    # Verifica o numero da chamada do sistema
    addi    t0, zero, 10
    beq     t0, a7, goToExit          # ecall exit
    addi    t0, zero, 110
    beq     t0, a7, goToExit          # ecall exit
    
    addi    t0, zero, 1               # ecall 1 = print int
    beq     t0, a7, goToPrintInt
    addi    t0, zero, 101             # ecall 1 = print int
    beq     t0, a7, goToPrintInt

    addi    t0, zero, 2               # ecall 2 = print float
    beq     t0, a7, goToPrintFloat
    addi    t0, zero, 102             # ecall 2 = print float
    beq     t0, a7, goToPrintFloat

    addi    t0, zero, 4               # ecall 4 = print string
    beq     t0, a7, goToPrintString
    addi    t0, zero, 104             # ecall 4 = print string
    beq     t0, a7, goToPrintString

    addi    t0, zero, 5               # ecall 5 = read int
    beq     t0, a7, goToReadInt
    addi    t0, zero, 105             # ecall 5 = read int
    beq     t0, a7, goToReadInt

    addi    t0, zero, 6               # ecall 6 = read float
    beq     t0, a7, goToReadFloat
    addi    t0, zero, 106             # ecall 6 = read float
    beq     t0, a7, goToReadFloat

    addi    t0, zero, 8               # ecall 8 = read string
    beq     t0, a7, goToReadString
    addi    t0, zero, 108             # ecall 8 = read string
    beq     t0, a7, goToReadString

    addi    t0, zero, 11              # ecall 11 = print char
    beq     t0, a7, goToPrintChar
    addi    t0, zero, 111             # ecall 11 = print char
    beq     t0, a7, goToPrintChar

    addi    t0, zero, 12              # ecall 12 = read char
    beq     t0, a7, goToReadChar
    addi    t0, zero, 112             # ecall 12 = read char
    beq     t0, a7, goToReadChar

    addi    t0, zero, 30              # ecall 30 = time
    beq     t0, a7, goToTime
    addi    t0, zero, 130             # ecall 30 = time
    beq     t0, a7, goToTime
    
    addi    t0, zero, 32              # ecall 32 = sleep
    beq     t0, a7, goToSleep
    addi    t0, zero, 132             # ecall 32 = sleep
    beq     t0, a7, goToSleep

    addi    t0, zero, 41              # ecall 41 = random
    beq     t0, a7, goToRandom
    addi    t0, zero, 141             # ecall 41 = random
    beq     t0, a7, goToRandom

    addi    t0, zero, 34       		# ecall 34 = print hex
    beq     t0, a7, goToPrintHex
    addi    t0, zero, 134		# ecall 34 = print hex
    beq     t0, a7, goToPrintHex
    
    addi    t0, zero, 31              # ecall 31 = MIDI out
    beq     t0, a7, goToMidiOut       # Generate tone and return immediately
    addi    t0, zero, 131             # ecall 31 = MIDI out
    beq     t0, a7, goToMidiOut

    addi    t0, zero, 33              # ecall 33 = MIDI out synchronous
    beq     t0, a7, goToMidiOutSync   # Generate tone and return upon tone completion
    addi    t0, zero, 133             # ecall 33 = MIDI out synchronous
    beq     t0, a7, goToMidiOutSync

    addi    t0, zero, 48              # ecall 48 = CLS
    beq     t0, a7, goToCLS
    addi    t0, zero, 148              # ecall 48 = CLS
    beq     t0, a7, goToCLS

    addi    t0, zero, 47              # ecall 47 = DrawLine
    beq     t0, a7, goToBRES
    addi    t0, zero, 147              # ecall 47 = DrawLine
    beq     t0, a7, goToBRES    
       

    addi    t0, zero, 36              # ecall 36 = PrintIntUnsigned
    beq     t0, a7, goToPrintIntUnsigned
    addi    t0, zero, 136             # ecall 36 = PrintIntUnsigned
    beq     t0, a7, goToPrintIntUnsigned


	## end execution ##
	goToExit:   	DE1(s8,goToExitDE2)		# se for a DE1 pula
			li 	a7, 10			# chama o ecall normal do Rars
			ecall				# exit ecall	
	goToExitDE2:	j 	goToExitDE2		# trava o processador : Não tem sistema operacional!

	goToPrintInt:	jal     printInt               	# chama printInt
			j       endEcall

	goToPrintString: jal     printString           	# chama printString
			 j       endEcall

	goToPrintChar:	jal     printChar		# chama printChar
			j       endEcall

	goToPrintFloat: NAOTEM_F(s8,NaoExisteEcall)
			jal     printFloat		# chama printFloat
			j       endEcall

	goToReadChar:	jal     readChar              	# chama readChar
			j       endEcall

	goToReadInt:   	jal     readInt                 # chama readInt
			j       endEcall

	goToReadString:	jal     readString              # chama readString
			j       endEcall

	goToReadFloat:	NAOTEM_F(s8,NaoExisteEcall)
			jal     readFloat               # chama readFloat
			j       endEcall

	goToPrintHex:	jal     printHex                # chama printHex
			j       endEcall

	goToPrintIntUnsigned: 	jal	printIntUnsigned	# chama Print Unsigned Int
				j	endEcall  
					
	goToMidiOut:	jal     midiOut                 # chama MIDIout
			j       endEcall

	goToMidiOutSync: jal     midiOutSync   		# chama MIDIoutSync
			 j       endEcall

	goToTime:	jal     Time                    # chama time
			j       endEcall

	goToSleep:	jal     Sleep                  	# chama sleep
			j       endEcall

	goToRandom:	jal     Random                 	# chama random
			j       endEcall

	goToCLS:	jal     clsCLS                 	# chama CLS
			j       endEcall

	goToBRES:	jal     BRESENHAM               # chama BRESENHAM
			j       endEcall    	
  		    				    		    				    		    		
		

endEcall:  	lw	x1,   0(sp)  # recupera QUASE todos os registradores na pilha
		lw	x2,   4(sp)	
		lw	x3,   8(sp)	
		lw	x4,  12(sp)      	
		lw	x5,  16(sp)      	
		lw	x6,  20(sp)	
		lw	x7,  24(sp)
		lw	x8,  28(sp)
		lw	x9,  32(sp)
	#	lw      x10, 36(sp)	# a0 retorno de valor
	#	lw     x11, 40(sp)	# a1 retorno de valor
		lw     x12, 44(sp)
		lw     x13, 48(sp)
		lw     x14, 52(sp)
		lw     x15, 56(sp)
		lw     x16, 60(sp)
		lw     x17, 64(sp)
		lw     x18, 68(sp)
		lw     x19, 72(sp)
		lw     x20, 76(sp)
		lw     x21, 80(sp)
		lw     x22, 84(sp)
		lw     x23, 88(sp)
		lw     x24, 92(sp)
		lw     x25, 96(sp)
		lw     x26, 100(sp)
		lw     x27, 104(sp)
		lw     x28, 108(sp)
		lw     x29, 112(sp)
		lw     x30, 116(sp)
		lw     x31, 120(sp)
		NAOTEM_F(s8,endEcall.pula)
		flw    f0,  124(sp)
		flw    f1,  128(sp)
		flw    f2,  132(sp)
		flw    f3,  136(sp)
		flw    f4,  140(sp)
		flw    f5,  144(sp)
		flw    f6,  148(sp)
		flw    f7,  152(sp)
		flw    f8,  156(sp)
		flw    f9,  160(sp)
	#   	flw    f10, 164(sp)		# fa0 retorno de valor
	#	flw    f11, 168(sp)		# fa1 retorno de valor
		flw    f12, 172(sp)
		flw    f13, 176(sp)
		flw    f14, 180(sp)
		flw    f15, 184(sp)
		flw    f16, 188(sp)
		flw    f17, 192(sp)
		flw    f18, 196(sp)
		flw    f19, 200(sp)
		flw    f20, 204(sp)
		flw    f21, 208(sp)
		flw    f22, 212(sp)
		flw    f23, 216(sp)
		flw    f24, 220(sp)
		flw    f25, 224(sp)
		flw    f26, 228(sp)
		flw    f27, 232(sp)
		flw    f28, 236(sp)
		flw    f29, 240(sp)
		flw    f30, 244(sp)
		flw    f31, 248(sp)

endEcall.pula:	addi    sp, sp, 264

		csrr 	tp, uepc 	# le o valor de EPC salvo no registrador uepc (reg 65)
		addi 	tp, tp, 4	# soma 4 para obter a instrucao seguinte ao ecall
		csrw 	tp, uepc	# coloca no registrador uepc
		uret			# retorna PC=uepc


#########################################################
#  Nao Existe Ecall                         		#
#  Para o caso de ecalls de fp mas ISA RV32I e RV32IM   #
#########################################################

NaoExisteEcall: addi 	a0, zero, 0xc0 		## printa tela de azul
		addi 	a1, zero, 0
		mv 	a6, a7
		addi 	a7, zero, 148
		jal 	clsCLS
  		la 	a0, CauseE
		li 	a1, 0
		li 	a2, 1
		li 	a3, 0x0000c0ff
		jal 	printString
		mv 	a0, a6
		li 	a1, 104
		li 	a2, 1
		li 	a3, 0x0000c0ff
		jal 	printInt
		csrr	a0,uepc
		li 	a1, 136
		li 	a2, 1
		li 	a3, 0x0000c0ff
		jal 	printHex		
		j 	goToExit

####################################################################################################

#############################################
#  PrintInt                                 #
#  a0    =    valor inteiro                 #
#  a1    =    x                             #
#  a2    =    y  			    #
#  a3    =    cor                           #
#############################################

printInt:	addi 	sp, sp, -4			# Aloca espaco
		sw 	ra, 0(sp)			# salva ra
		la 	t0, TempBuffer			# carrega o Endereco do Buffer da String
		
		bge 	a0, zero, ehposprintInt		# Se eh positvo
		li 	t1, '-'				# carrega o sinal -
		sb 	t1, 0(t0)			# coloca no buffer
		addi 	t0, t0, 1			# incrementa endereco do buffer
		sub 	a0, zero, a0			# torna o numero positivo
		
ehposprintInt:  li 	t2, 10				# carrega numero 10
		li 	t1, 0				# carrega numero de digitos com 0
		
loop1printInt:	TEM_M(s8,printInt.pula1)
		DIV10(t4,a0)
		REM10(t3,a0)
		j 	printInt.pula1d
printInt.pula1:	div 	t4, a0, t2			# divide por 10 (quociente)
		rem 	t3, a0, t2			# resto
printInt.pula1d:addi 	sp, sp, -4			# aloca espaco na pilha
		sw 	t3, 0(sp)			# coloca resto na pilha
		mv 	a0, t4				# atualiza o numero com o quociente
		addi 	t1, t1, 1			# incrementa o contador de digitos
		bne 	a0, zero, loop1printInt		# verifica se o numero eh zero
				
loop2printInt:	lw 	t2, 0(sp)			# le digito da pilha
		addi 	sp, sp, 4			# libera espaco
		addi 	t2, t2, 48			# converte o digito para ascii
		sb 	t2, 0(t0)			# coloca caractere no buffer
		addi 	t0, t0, 1			# incrementa endereco do buffer
		addi 	t1, t1, -1			# decrementa contador de digitos
		bne 	t1, zero, loop2printInt		# eh o ultimo?
		sb 	zero, 0(t0)			# insere \NULL na string
		
		la 	a0, TempBuffer			# Endereco do buffer da srting
		jal 	printString			# chama o print string
				
		lw 	ra, 0(sp)			# recupera a
		addi 	sp, sp, 4			# libera espaco
fimprintInt:	ret					# retorna
		


#############################################
#  PrintHex                                 #
#  a0    =    valor inteiro                 #
#  a1    =    x                             #
#  a2    =    y                             #
#  a3    =    cor			    #
#############################################

printHex:	addi    sp, sp, -4    		# aloca espaco
    		sw      ra, 0(sp)		# salva ra
		mv 	t0, a0			# Inteiro de 32 bits a ser impresso em Hexa
		la 	t1, TabelaHexASCII	# endereco da tabela HEX->ASCII
		la 	t2, TempBuffer		# onde a string sera montada

		li 	t3,'0'			# Caractere '0'
		sb 	t3,0(t2)		# Escreve '0' no Buffer da String
		li 	t3,'x'			# Caractere 'x'
		sb 	t3,1(t2)		# Escreve 'x' no Buffer da String
		addi 	t2,t2,2			# novo endereco inicial da string

		li 	t3, 28			# contador de nibble   inicio = 28
loopprintHex:	blt 	t3, zero, fimloopprintHex	# terminou? t3<0?
		srl 	t4, t0, t3		# desloca o nibble para direita
		andi 	t4, t4, 0x000F		# mascara o nibble	
		add 	t4, t1, t4		# endereco do ascii do nibble
		lb 	t4, 0(t4)		# le ascii do nibble
		sb 	t4, 0(t2)		# armazena o ascii do nibble no buffer da string
		addi 	t2, t2, 1		# incrementa o endereco do buffer
		addi 	t3, t3, -4		# decrementa o numero do nibble
		j 	loopprintHex
		
fimloopprintHex: sb 	zero,0(t2)		# grava \null na string
		la 	a0, TempBuffer		# Argumento do print String
    		jal	printString		# Chama o print string
    			
		lw 	ra, 0(sp)		# recupera ra
		addi 	sp, sp, 4		# libera espaco
fimprintHex:	ret				# retorna


#####################################
#  PrintSring                       #
#  a0    =  endereco da string      #
#  a1    =  x                       #
#  a2    =  y                       #
#  a3    =  cor		    	    #
#####################################

printString:	addi	sp, sp, -8			# aloca espaco
    		sw	ra, 0(sp)			# salva ra
    		sw	s0, 4(sp)			# salva s0
    		mv	s0, a0              		# s0 = endereco do caractere na string

loopprintString:lb	a0, 0(s0)                 	# le em a0 o caracter a ser impresso

    		beq     a0, zero, fimloopprintString	# string ASCIIZ termina com NULL

    		jal     printChar       		# imprime char
    		
		addi    a1, a1, 8                 	# incrementa a coluna
		li 	t6, 313		
		blt	a1, t6, NaoPulaLinha	    	# se ainda tiver lugar na linha
    		addi    a2, a2, 8                 	# incrementa a linha
    		mv    	a1, zero			# volta a coluna zero

NaoPulaLinha:	addi    s0, s0, 1			# proximo caractere
    		j       loopprintString       		# volta ao loop

fimloopprintString:	lw      ra, 0(sp)    		# recupera ra
			lw 	s0, 0(sp)		# recupera s0 original
    			addi    sp, sp, 8		# libera espaco
fimprintString:	ret      	    			# retorna


#########################################################
#  PrintChar                                            #
#  a0 = char(ASCII)                                     #
#  a1 = x                                               #
#  a2 = y                                               #
#  a3 = cores (0x0000bbff) 	b = fundo, f = frente	#
#  a4 = frame (0 ou 1)					#
#########################################################
#   t0 = i                                             #
#   t1 = j                                             #
#   t2 = endereco do char na memoria                   #
#   t3 = metade do char (2a e depois 1a)               #
#   t4 = endereco para impressao                       #
#   t5 = background color                              #
#   t6 = foreground color                              #
#########################################################
#	t9 foi convertido para s9 pois nao ha registradores temporarios sobrando dentro desta funcao


printChar:	li 	t4, 0xFF	# t4 temporario
		slli 	t4, t4, 8	# t4 = 0x0000FF00 (no RARS, nao podemos fazer diretamente "andi rd, rs1, 0xFF00")
		and    	t5, a3, t4   	# t5 obtem cor de fundo
    		srli	t5, t5, 8	# numero da cor de fundo
		andi   	t6, a3, 0xFF    # t6 obtem cor de frente

		li 	tp, ' '
		blt 	a0, tp, printChar.NAOIMPRIMIVEL	# ascii menor que 32 nao eh imprimivel
		li 	tp, '~'
		bgt	a0, tp, printChar.NAOIMPRIMIVEL	# ascii Maior que 126  nao eh imprimivel
    		j       printChar.IMPRIMIVEL
    
printChar.NAOIMPRIMIVEL: li      a0, 32		# Imprime espaco

printChar.IMPRIMIVEL:	li	tp, NUMCOLUNAS		# Num colunas 320
			TEM_M(s8,printChar.mul1)
			MULTIPLY(t4,tp,a2)
			j printChar.mul1d
printChar.mul1:		mul     t4, tp, a2			# multiplica a2x320  t4 = coordenada y
printChar.mul1d:	add     t4, t4, a1               	# t4 = 320*y + x
			addi    t4, t4, 7                 	# t4 = 320*y + (x+7)
			li      tp, VGAADDRESSINI0          	# Endereco de inicio da memoria VGA0
			beq 	a4, zero, printChar.PULAFRAME		# Verifica qual o frame a ser usado em a4
			li      tp, VGAADDRESSINI1          	# Endereco de inicio da memoria VGA1
printChar.PULAFRAME:	add     t4, t4, tp               	# t4 = endereco de impressao do ultimo pixel da primeira linha do char
			addi    t2, a0, -32               	# indice do char na memoria
			slli    t2, t2, 3                 	# offset em bytes em relacao ao endereco inicial
			la      t3, LabelTabChar		# endereco dos caracteres na memoria
			add     t2, t2, t3               	# endereco do caractere na memoria
			lw      t3, 0(t2)                 	# carrega a primeira word do char
			li 	t0, 4				# i=4

printChar.forChar1I:	beq     t0, zero, printChar.endForChar1I # if(i == 0) end for i
    			addi    t1, zero, 8               	# j = 8

printChar.forChar1J:	beq     t1, zero, printChar.endForChar1J # if(j == 0) end for j
        		andi    s9, t3, 0x001			# primeiro bit do caracter
        		srli    t3, t3, 1             		# retira o primeiro bit
        		beq     s9, zero, printChar.printCharPixelbg1	# pixel eh fundo?
        		sb      t6, 0(t4)             		# imprime pixel com cor de frente
        		j       printChar.endCharPixel1
printChar.printCharPixelbg1:	sb      t5, 0(t4)                # imprime pixel com cor de fundo
printChar.endCharPixel1: addi    t1, t1, -1                	# j--
    			addi    t4, t4, -1                	# t4 aponta um pixel para a esquerda
    			j       printChar.forChar1J		# vollta novo pixel

printChar.endForChar1J: addi    t0, t0, -1 		# i--
    			addi    t4, t4, 328           	# 2**12 + 8
    			j       printChar.forChar1I	# volta ao loop

printChar.endForChar1I:	lw      t3, 4(t2)           	# carrega a segunda word do char
			li 	t0, 4			# i = 4
printChar.forChar2I:    beq     t0, zero, printChar.endForChar2I  # if(i == 0) end for i
    			addi    t1, zero, 8             # j = 8

printChar.forChar2J:	beq	t1, zero, printChar.endForChar2J # if(j == 0) end for j
        		andi    s9, t3, 0x001	    		# pixel a ser impresso
        		srli    t3, t3, 1                 	# desloca para o proximo
        		beq     s9, zero, printChar.printCharPixelbg2 # pixel eh fundo?
        		sb      t6, 0(t4)			# imprime cor frente
        		j       printChar.endCharPixel2		# volta ao loop

printChar.printCharPixelbg2:	sb      t5, 0(t4)		# imprime cor de fundo

printChar.endCharPixel2:	addi    t1, t1, -1		# j--
    				addi    t4, t4, -1              # t4 aponta um pixel para a esquerda
    				j       printChar.forChar2J

printChar.endForChar2J:	addi	t0, t0, -1 		# i--
    			addi    t4, t4, 328		#
    			j       printChar.forChar2I	# volta ao loop

printChar.endForChar2I:	ret				# retorna


#########################################
# ReadChar           			#
# a0 = valor ascii da tecla   		#
# 2017/2  				#
######################################### 

readChar: 		nop
#DE1(s8,readCharDE2)   # Eh necessario mesmo???

##### Tratamento para uso com o Keyboard Display MMIO Tool do Rars
readCharKDMMIO:		li 	t0, KDMMIO_Ctrl			# Execucao com Polling do KD MMIO

loopReadCharKDMMIO:  	lw     	a0, 0(t0)   			# le o bit de flag do teclado
			andi 	a0, a0, 0x0001			# mascara bit 0
			beqz    a0, loopReadCharKDMMIO  	# testa se uma tecla foi pressionada
   			lw 	a0, 4(t0)			# le o ascii da tecla pressionada
			j fimreadChar				# fim Read Char

					
												
##### Tratamento para uso com o teclado PS2 da DE2 usando Buffer0 teclado
#### muda a0, t0,t1,t2,t3 e s0
#### Cuidar: ao entrar s0 ja deve conter o endereco la s0,LabelScanCode  #####
readCharDE2:  	li      t0, Buffer0Teclado 			# Endereco buffer0
    		lw     	t1, 0(t0)				# conteudo inicial do buffer
	
loopReadChar:  	lw     	t2, 0(t0)   				# le buffer teclado
		bne     t2, t1, buffermodificadoChar    	# testa se o buffer foi modificado

atualizaBufferChar:  mv t1, t2			# atualiza o buffer com o novo valor
    		j       loopReadChar		# loop de principal de leitura 

buffermodificadoChar:	li t5, 0xFF
	slli 	t5, t5, 8			# t5 = 0x0000FF00
	and    	t3, t2, t5 			# mascara o 2o scancode
	li 	tp, 0x0000F000
	beq     t3, tp, teclasoltaChar		# eh 0xF0 no 2o scancode? tecla foi solta
	li	tp, 0x000000FF
	and	t3, t2, tp			# mascara 1o scancode	(essa podemos fazer diretamente)
	li	tp, 0x00000012
    	bne 	t3, tp, atualizaBufferChar	# nao eh o SHIFT que esta pressionado ? volta a ler 
	la      s0, LabelScanCodeShift		# se for SHIFT que esta pressionado atualiza o endereco da tabel
    	j       atualizaBufferChar		# volta a ler

teclasoltaChar:		andi t3, t2, 0x00FF		# mascara o 1o scancode
	li	tp, 0x00000080
  	bgt	t3, tp, atualizaBufferChar		# se o scancode for > 0x80 entao nao eh imprimivel!
  	li	tp, 0x00000012
	bne 	t3, tp, naoehshiftChar			# nao foi o shift que foi solto? entao processa
	la 	s0, LabelScanCode			# shift foi solto atualiza o endereco da tabela
	j 	atualizaBufferChar			# volta a ler
	
naoehshiftChar:	   	add     t3, s0, t3              # endereco na tabela de scancode da tecla com ou sem shift
    	lb      a0, 0(t3)				# le o ascii do caracter para a0
    	beq     a0, zero, atualizaBufferChar		# se for caractere nao imprimivel volta a ler
    	
fimreadChar: 	ret			# retorna
	
#########################################
#    ReadString         	 	#
# a0 = end Inicio      	 		#
# a1 = tam Max String 		 	#
# a2 = end do ultimo caractere	 	#
# a3 = num de caracteres digitados	#
# 2018/1     2019/2           		#
#########################################
# muda a2, a3, s2 e s0  

readString: 	addi 	sp, sp, -8			# reserva espaco na pilha
		sw 	s0, 4(sp)			# salva s0
		sw 	ra, 0(sp)			# salva ra
		li 	a3, 0				# zera o contador de caracteres digitados
		mv 	s2, a0				# salva o endereco inicial
    		la      s0, LabelScanCode      		# Endereco da tabela de scancode inicial para readChar
    		
loopreadString: beq 	a1, a3, fimreadString   	# buffer cheio fim
	
		addi 	sp, sp, -8
		sw 	ra, 0(sp)			# salva ra
		sw 	a0, 4(sp)			# salva a0 pois ele sera reescrito em readChar
		jal 	readChar			# le um caracter do teclado (retorno em a0)
		mv 	t6, a0				# t6 eh a letra lida em readChar
		lw 	ra, 0(sp)
		lw 	a0, 4(sp)
		addi 	sp, sp, 8

		li 	tp, 0x08			
		bne	t6, tp, PulaBackSpace		# Se nao for BACKSPACE
		beq	zero, a3, loopreadString	# Se não tem nenhum caractere no buffer apenas volta a ler
		addi	a3, a3, -1			# diminui contador
		addi 	a0, a0, -1			# diminui endereco do buffer
		sb 	zero, 0(a0)			# coloca zero no caractere anterior
		j loopreadString
		
PulaBackSpace:	li	tp, 0x0A
		beq 	t6, tp, fimreadString		# se for tecla ENTER fim
		sb 	t6, 0(a0)			# grava no buffer
		addi 	a3, a3, 1			# incrementa contador
		addi 	a0, a0, 1			# incrementa endereco no buffer
		j loopreadString			# volta a ler outro caractere
	
fimreadString: 	sb 	zero, 0(a0)			# grava NULL no buffer
		addi 	a2, a0, -1			# Para que a2 tenha o endereco do ultimo caractere digitado
		mv	a0, s2				# a0 volta a ter o endereco inicial da string
		lw 	ra, 0(sp)			# recupera ra
		lw	s0, 4(sp)			# recupera s0
		addi 	sp, sp, 8			# libera espaco
		ret					# retorna
	
	
###########################
#    ReadInt              #
# a0 = valor do inteiro   #
#                         #
###########################

readInt: 	addi 	sp,sp,-4		# reserva espaco na pilha
	sw 	ra, 0(sp)			# salva ra
	la 	a0, TempBuffer			# Endereco do buffer de string
	li 	a1, 10				# numero maximo de digitos
	jal 	readString			# le uma string de ate 10 digitos, a3 numero de digitos
	mv 	t0, a2				# copia endereco do ultimo digito
	li 	t2, 10				# dez
	li 	t3, 1				# dezenas, centenas, etc
	mv 	a0, zero			# zera o numero
	
loopReadInt: 	beq	a3,zero, fimReadInt	# Leu todos os digitos
	lb 	t1, (t0)			# le um digito
	li	tp, 0x0000002D
	beq 	t1, tp, ehnegReadInt		# = '-'
	li	tp, 0x0000002B
	beq 	t1, tp, ehposReadInt		# = '+'
	li	tp, 0x00000030
	blt 	t1, tp, naoehReadInt		# <'0'
	li	tp, 0x00000039
	bgt 	t1, tp, naoehReadInt		# >'9'
	addi 	t1, t1, -48			# transforma ascii em numero
	TEM_M(s8,readInt.mul1)
	MULTIPLY(t1,t1,t3)
	j readInt.mul1d
readInt.mul1: 	mul 	t1, t1, t3			# multiplica por dezenas/centenas
readInt.mul1d:	add 	a0, a0, t1			# soma no numero
	TEM_M(s8,readInt.mul2)
	MULTIPLY(t3,t3,t2)
	j readInt.mul2d
readInt.mul2: 	mul 	t3, t3, t2			# proxima dezena/centena
readInt.mul2d:	addi 	t0, t0, -1			# busca o digito anterior
	addi	a3, a3, -1			# reduz o contador de digitos 
	j 	loopReadInt			# volta para buscar proximo digito

naoehReadInt:	#j instructionException		# gera erro "instrucao invalida"
		j fimReadInt			# como nao esta implmentado apenas retorna

ehnegReadInt:	sub a0,zero,a0			# se for negativo

ehposReadInt:					# se for positivo so retorna

fimReadInt:	lw 	ra, 0(sp)		# recupera ra
		addi 	sp, sp, 4		# libera espaco
		ret				# fim ReadInt


###########################################
#        MidiOut 31 (2015/1)              #
#  a0 = pitch (0-127)                     #
#  a1 = duration in milliseconds          #
#  a2 = instrument (0-15)                 #
#  a3 = volume (0-127)                    #
###########################################


#################################################################################################
#
# Note Data           = 32 bits     |   1'b - Melody   |   4'b - Instrument   |   7'b - Volume   |   7'b - Pitch   |   1'b - End   |   1'b - Repeat   |   11'b - Duration   |
#
# Note Data (ecall) = 32 bits     |   1'b - Melody   |   4'b - Instrument   |   7'b - Volume   |   7'b - Pitch   |   13'b - Duration   |
#
#################################################################################################
midiOut:
	DE1(s8,midiOutDE2)
	
	li a7,31		# Chama o ecall normal
	ecall
	j fimmidiOut

midiOutDE2:	li      t0, NoteData
    		add     t1, zero, zero

    		# Melody = 0

    		# Definicao do Instrumento
   	 	andi    t2, a2, 0x0000000F
    		slli    t2, t2, 27
    		or      t1, t1, t2

    		# Definicao do Volume
    		andi    t2, a3, 0x0000007F
    		slli    t2, t2, 20
    		or      t1, t1, t2

    		# Definicao do Pitch
    		andi    t2, a0, 0x0000007F
    		slli    t2, t2, 13
    		or      t1, t1, t2

    		# Definicao da Duracao
		li 	t4, 0x1FF
		slli 	t4, t4, 4
		addi 	t4, t4, 0x00F			# t4 = 0x00001FFF
    		and    	t2, a1, t4
    		or      t1, t1, t2

    		# Guarda a definicao da duracao da nota na Word 1
    		j       SintMidOut

SintMidOut:	sw	t1, 0(t0)

	    		# Verifica a subida do clock AUD_DACLRCK para o sintetizador receber as definicoes
	    		li      t2, NoteClock
Check_AUD_DACLRCK:     	lw      t3, 0(t2)
    			beq     t3, zero, Check_AUD_DACLRCK

fimmidiOut:    		ret

###########################################
#        MidiOut 33 (2015/1)              #
#  a0 = pitch (0-127)                     #
#  a1 = duration in milliseconds          #
#  a2 = instrument (0-127)                #
#  a3 = volume (0-127)                    #
###########################################

#################################################################################################
#
# Note Data             = 32 bits     |   1'b - Melody   |   4'b - Instrument   |   7'b - Volume   |   7'b - Pitch   |   1'b - End   |   1'b - Repeat   |   8'b - Duration   |
#
# Note Data (ecall)   	= 32 bits     |   1'b - Melody   |   4'b - Instrument   |   7'b - Volume   |   7'b - Pitch   |   13'b - Duration   |
#
#################################################################################################
midiOutSync:
	DE1(s8,midiOutSyncDE2)
	
	li a7,33		# Chama o ecall normal
	ecall
	j fimmidiOutSync
	
midiOutSyncDE2:	li      t0, NoteData
    		add     t1, zero, zero

    		# Melody = 1
    		lui    	t1, 0x08000
		slli	t1,t1,4
		
    		# Definicao do Instrumento
    		andi    t2, a2, 0x00F
    		slli    t2, t2, 27
    		or      t1, t1, t2

    		# Definicao do Volume
    		andi    t2, a3, 0x07F
    		slli    t2, t2, 20
    		or      t1, t1, t2

    		# Definicao do Pitch
    		andi    t2, a0, 0x07F
    		slli    t2, t2, 13
    		or      t1, t1, t2

    		# Definicao da Duracao
		li 	t4, 0x1FF
		slli 	t4, t4, 4
		addi 	t4, t4, 0x00F			# t4 = 0x00001FFF
    		and    	t2, a1, t4
    		or      t1, t1, t2

    		# Guarda a definicao da duracao da nota na Word 1
    		j       SintMidOutSync

SintMidOutSync:	sw	t1, 0(t0)

    		# Verifica a subida do clock AUD_DACLRCK para o sintetizador receber as definicoes
    		li      t2, NoteClock
    		li      t4, NoteMelody

Check_AUD_DACLRCKSync:	lw      t3, 0(t2)
    			beq     t3, zero, Check_AUD_DACLRCKSync

Melody:     	lw      t5, 0(t4)
    		bne     t5, zero, Melody

fimmidiOutSync:	ret


#################################
# printFloat                    #
# imprime Float em fa0          #
# na posicao (a1,a2)	cor a3  #
#################################
# muda s0, s1

printFloat:	addi 	sp, sp, -4
		sw 	ra, 0(sp)				# salva ra
		la 	s0, TempBuffer

		# Encontra o sinal do numero e coloca no Buffer
		li 	t0, '+'			# define sinal '+'
		fmv.x.s s1, fa0			# recupera o numero float sem conversao
		srli	s1, s1, 31		# bit 31(sinal) em bit 0, numero eh negativo s1=1
		beq 	s1, zero, ehposprintFloat	# eh positivo s1=0
		li 	t0, '-'				# define sinal '-'
ehposprintFloat: sb 	t0, 0(s0)			# coloca sinal no buffer
		addi 	s0, s0, 1			# incrementa o endereco do buffer

		# Encontra o expoente em t0
		 fmv.x.s t0, fa0			# recupera o numero float sem conversao
		 lui	t1, 0x7F800
		 and 	t0, t0, t1   			# mascara com 0111 1111 1000 0000 0000 0000...
		 slli 	t0, t0, 1			# tira o sinal do numero
		 srli 	t0, t0, 24			# recupera o expoente

		# Encontra a fracao em t1
		fmv.x.s t1, fa0				# recupera o numero float sem conversao
		li 	t2, 0x007FFFFF			# t2 = 0x007FFFFF
		and 	t1, t1, t2			# mascara com 0000 0000 0111 1111 1111... 		 
			 
		beq 	t0, zero, ehExp0printFloat	# Expoente = 0
		li	tp, 0x000000FF			# TP = 255
		beq 	t0, tp, ehExp255printFloat	# Expoente = 255
		
		# Eh um numero float normal  t0 eh o expoente e t1 eh a fracao!
		# Encontra o E tal que 10^E <= x <10^(E+1)
		fabs.s 		ft0, fa0		# ft0 recebe o modulo  de x
		li		tp, 1
		fcvt.s.w 	ft1, tp			# ft1 recebe o numero 1.0
		li		tp, 10
		fcvt.s.w 	ft6, tp			# ft6 recebe o numero 10.0
		li		tp, 2
		fcvt.s.w 	ft8, tp
		fdiv.s		ft7, ft1, ft8		# ft7 recebe o numero 0.5
		
		flt.s 	t4, ft0, ft1		# ft0 < 1.0 ? Se sim, E deve ser negativo
		bnez	t4, menor1printFloat	# se a comparacao deu true (1), pula
		fmv.s 	ft2, ft6		# ft2  fator de multiplicacao = 10
		j 	cont2printFloat		# vai para expoente positivo
menor1printFloat: fdiv.s ft2,ft1,ft6		# ft2 fator multiplicativo = 0.1

			# calcula o expoente negativo de 10
cont1printFloat: 	fmv.s 	ft4, ft0			# inicia com o numero x 
		 	fmv.s 	ft3, ft1			# contador comeca em 1
loop1printFloat: 	fdiv.s 	ft4, ft4, ft2			# divide o numero pelo fator multiplicativo
		 	fle.s 	t3, ft4, ft1			# o numero eh > que 1? entao fim
		 	beq 	t3,zero, fimloop1printFloat
		 	fadd.s 	ft3, ft3, ft1			# incrementa o contador
		 	j 	loop1printFloat			# volta ao loop
		 	
fimloop1printFloat: 	fdiv.s 	ft4, ft4, ft2			# ajusta o numero
		 	j 	intprintFloat			# vai para imprimir a parte inteira

			# calcula o expoente positivo de 10
cont2printFloat:	fmv.s 	 ft4, ft0			# inicia com o numero x 
		 	fcvt.s.w ft3, zero			# contador comeca em 0
loop2printFloat:  	flt.s 	 t3, ft4, ft6			# resultado eh < que 10? entao fim
			fdiv.s 	 ft4, ft4, ft2			# divide o numero pelo fator multiplicativo
			bne 	 t3, zero, intprintFloat
		 	fadd.s 	 ft3, ft3, ft1			# incrementa o contador
		 	j 	 loop2printFloat

		# Neste ponto tem-se em t4 se ft0<1, em ft3 o expoente de 10 e ft0 0 modulo do numero e s1 o sinal
		# e em ft4 um numero entre 1 e 10 que multiplicado por Ef3 deve voltar ao numero		
		
	  		# imprime parte inteira (o sinal ja esta no buffer)
intprintFloat:		fmul.s 		ft4, ft4, ft2		# ajusta o numero
			fsub.s		ft4, ft4, ft7		# tira 0.5, dessa forma sempre ao converter estaremos fazendo floor
		  	fcvt.w.s	t0, ft4			# coloca floor de ft4 em t0
			fadd.s		ft4, ft4, ft7		# readiciona 0.5
			bnez		t0, pulaeh1print	# para corrigir multiplos inteiros de 10!
			li 		t0, 1
pulaeh1print:		addi 		t0, t0, 48		# converte para ascii			
			sb 		t0, 0(s0)		# coloca no buffer
		  	addi 		s0, s0, 1		# incrementta o buffer
		  
		  	# imprime parte fracionaria
		  	li 	t0, '.'				# carrega o '.'
		  	sb 	t0, 0(s0)			# coloca no buffer
		  	addi 	s0, s0, 1			# incrementa o buffer
		  
		  	# ft4 contem a mantissa com 1 casa nao decimal
		  	li 		t1, 8				# contador de digitos  -  8 casas decimais
loopfracprintFloat:  	beq 		t1, zero, fimfracprintFloat	# fim dos digitos?
			fsub.s		ft4, ft4, ft7			# tira 0.5
			fcvt.w.s 	t5, ft4				# floor de ft4
			fadd.s		ft4, ft4, ft7			# readiciona 0.5
			fcvt.s.w	ft5, t5				# reconverte em float so com a parte inteira
		  	fsub.s 		ft5, ft4, ft5			# parte fracionaria
		  	fmul.s 		ft5, ft5, ft6			# mult x 10
			fsub.s		ft5, ft5, ft7			# tira 0.5
			fcvt.w.s	t0, ft5				# coloca floor de ft5 em 10
		  	addi 		t0, t0, 48			# converte para ascii
		  	
			li 		tp, 48
			blt		t0, tp, pulaprtFloat1	# testa se eh menor que '0'
			li		tp, 57
			ble		t0, tp, pulaprtFloat2	# testa se eh menor ou igual que '9'
pulaprtFloat1:		li		t0, 48			# define como '0'		  			  	
		  	
pulaprtFloat2:	  	sb 		t0, 0(s0)			# coloca no buffer
		  	addi 		s0, s0, 1			# incrementa endereco
		  	addi 		t1, t1, -1			# decrementa contador
			fadd.s		ft5, ft5, ft7			# reincrementa 0.5
		  	fmv.s 		ft4, ft5			# coloca o numero em ft4
		  	j 		loopfracprintFloat		# volta ao loop
		  
		  	# imprime 'E'		  
fimfracprintFloat: 	li 	t0,'E'			# carrega 'E'
			sb 	t0, 0(s0)		# coloca no buffer
			addi 	s0, s0, 1		# incrementa endereco
			
		  	# imprime sinal do expoente
		  	li 	t0, '+'				# carrega '+'
		  	beqz 	t4, expposprintFloat		# nao eh negativo?
		  	li 	t0, '-'				# carrega '-'
expposprintFloat: 	sb 	t0, 0(s0)			# coloca no buffer
		  	addi 	s0, s0, 1			#incrementa endereco
				    
		  	# imprimeo expoente com 2 digitos (maximo E+38)
			li 	t1, 10				# carrega 10	
			fcvt.w.s  tp, ft3			# passa ft3 para t0
			div 	t0, tp, t1			# divide por 10 (dezena)
			rem	t2, tp, t1			# t0 = quociente, t2 = resto
			addi 	t0, t0, 48			# converte para ascii
			sb 	t0, 0(s0)			# coloca no buffer
			addi 	t2, t2, 48			# converte para ascii
			sb 	t2, 1(s0)			# coloca no buffer
			sb 	zero, 2(s0)			# insere \NULL da string
			la 	a0, TempBuffer			# endereco do Buffer										
	  		j 	fimprintFloat			# imprime a string
								
ehExp0printFloat: 	beq 	t1, zero, eh0printFloat		# Verifica se eh zero
		
ehDesnormprintFloat: 	la 	a0, NumDesnormP			# string numero desnormalizado positivo
			beq 	s1, zero, fimprintFloat		# o sinal eh 1? entao eh negativo
		 	la 	a0, NumDesnormN			# string numero desnormalizado negativo
			j 	fimprintFloat			# imprime a string

eh0printFloat:		la 	a0, NumZero			# string do zero
			j 	fimprintFloat 	 		# imprime a string
		 		 		 		 
ehExp255printFloat: 	beq 	t1, zero, ehInfprintFloat	# se mantissa eh zero entao eh Infinito

ehNaNprintfFloat:	la 	a0, NumNaN			# string do NaN
			j 	fimprintFloat			# imprime string

ehInfprintFloat:	la 	a0, NumInfP			# string do infinito positivo
			beq 	s1, zero, fimprintFloat		# o sinal eh 1? entao eh negativo
			la 	a0, NumInfN			# string do infinito negativo
								# imprime string
		
fimprintFloat:		jal 	printString			# imprime a string em a0
			lw 	ra, 0(sp)			# recupera ra
			addi 	sp, sp, 4			# libera espaco
			ret					# retorna


#################################
# readFloat       	 	#
# fa0 = float digitado        	#
# 2017/2			#
#################################

readFloat: addi sp, sp, -4			# aloca espaco
	sw 	ra, 0(sp)			# salva ra
	la 	a0, TempBuffer			# endereco do FloatBuffer
	li 	a1, 32				# numero maximo de caracteres
	jal	readString			# le string, retorna a2 ultimo endereco e a3 numero de caracteres
	mv 	s0, a2				# ultimo endereco da string (antes do \0)
	mv 	s1, a3				# numero de caracteres digitados
	la	s7, TempBuffer			# Endereco do primeiro caractere
	
lePrimeiroreadFloat:	mv 	t0, s7		# Endereco de Inicio
	lb 	t1, 0(t0)				# le primeiro caractere
	li	tp, 'e'					# TP = 101 = 'e'
	beq 	t1, tp, insere0AreadFloat		# insere '0' antes
	li 	tp, 'E'					# TP = 69 = 'E'
	beq 	t1, tp, insere0AreadFloat		# insere '0' antes
	li	tp, '.'					# TP = 46 = '.'
	beq 	t1, tp, insere0AreadFloat		#  insere '0' antes
	li	tp, '+'					# TP = 43 = '+'
	beq 	t1, tp, pulaPrimreadChar		# pula o primeiro caractere
	li	tp, '-'					# TP = 45 = '-'
	beq 	t1, tp, pulaPrimreadChar
	j leUltimoreadFloat

pulaPrimreadChar: addi s7,s7,1			# incrementa o endereco inicial
		  j lePrimeiroreadFloat		# volta a testar o novo primeiro caractere
		  
insere0AreadFloat: mv t0, s0			# endereco do ultimo caractere
		   addi s0, s0, 1		# desloca o ultimo endereco para o proximo
	   	   addi s1, s1, 1		# incrementa o num. caracteres
	   	   sb 	zero, 1(s0)		# \NULL do final de string
	   	   mv t5, s7			# primeiro caractere
insere0Aloop:	   beq 	t0, t5, saiinsere0AreadFloat	# chegou no inicio entao fim
		   lb 	t1, 0(t0)		# le caractere
		   sb 	t1, 1(t0)		# escreve no proximo
		   addi t0, t0, -1		# decrementa endereco
		   j insere0Aloop		# volta ao loop
saiinsere0AreadFloat: li t1, '0'		# ascii '0'
		   sb t1, 0(t0)			# escreve '0' no primeiro caractere

leUltimoreadFloat: lb  	t1, 0(s0)			# le ultimo caractere
		li	tp, 'e'				# TP = 101 = 'e'
		beq 	t1, tp, insere0PreadFloat	# insere '0' depois
		li 	tp, 'E'				# TP = 69 = 'E'
		beq 	t1, tp, insere0PreadFloat	# insere '0' depois
		li	tp, '.'				# TP = 46 = '.'
		beq 	t1, tp, insere0PreadFloat	# insere '0' depois
		j 	inicioreadFloat
	
insere0PreadFloat: addi	s0, s0, 1		# desloca o ultimo endereco para o proximo
	   	   addi	s1, s1, 1		# incrementa o num. caracteres
		   li 	t1,'0'			# ascii '0'
		   sb 	t1,0(s0)		# escreve '0' no ultimo
		   sb 	zero,1(s0)		# \null do final de string

inicioreadFloat:  fcvt.s.w 	fa0, zero	# fa0 Resultado inicialmente zero
		li 	t0, 10			# inteiro 10	
		fcvt.s.w 	ft6, t0		# ft6 contem sempre o numero cte 10.0000
		li 	t0, 1			# inteiro 1
		fcvt.s.w 	ft1, t0		# ft1 contem sempre o numero cte 1.0000	
	
##### Verifica se tem 'e' ou 'E' na string  resultado em s3			
procuraEreadFloat:	addi 	s3, s0, 1			# inicialmente nao tem 'e' ou 'E' na string (fora da string)
			mv 	t0, s7				# endereco inicial
loopEreadFloat: 	beq 	t0, s0, naotemEreadFloat	# sai se nao encontrou 'e'
			lb 	t1, 0(t0)			# le o caractere
			li	tp, 'e'				# TP = 101 = 'e'
			beq 	t1, tp, ehEreadFloat		# tem 'e'
			li 	tp, 'E'				# TP = 69 = 'E'
			beq	t1, tp, ehEreadFloat		# tem 'E'
			addi 	t0, t0, 1			# incrementa endereco
			j 	loopEreadFloat			# volta ao loop
ehEreadFloat: 		mv 	s3, t0				# endereco do 'e' ou 'E' na string
naotemEreadFloat:						# nao tem 'e' ou 'E' s3 eh o endereco do \0 da string

##### Verifica se tem '.' na string resultado em s2 espera-se que nao exista ponto no expoente
procuraPontoreadFloat:	mv 	s2, s3				# local inicial do ponto na string (='e' se existir) ou fora da string	
			mv 	t0, s7				# endereco inicial
loopPontoreadFloat: 	beq 	t0, s0, naotemPontoreadFloat	# sai se nao encontrou '.'
			lb 	t1, 0(t0)			# le o caractere
			li	tp, '.'				# TP = 46 = '.'
			beq 	t1, tp, ehPontoreadFloat	# tem '.'
			addi 	t0, t0, 1			# incrementa endereco
			j 	loopPontoreadFloat		# volta ao loop
ehPontoreadFloat: 	mv 	s2, t0				# endereco do '.' na string
naotemPontoreadFloat:						# nao tem '.' s2 = local do 'e' ou \0 da string

### Encontra a parte inteira em fa0
intreadFloat:		fcvt.s.w 	ft2, zero		# zera parte inteira
			addi 	t0, s2, -1			# endereco do caractere antes do ponto
			fmv.s 	ft3, ft1			# ft3 contem unidade/dezenas/centenas		
			mv 	t5, s7				# Primeiro Endereco
loopintreadFloat: 	blt 	t0, t5, fimintreadFloat		# sai se o endereco for < inicio da string
			lb 	t1, 0(t0)			# le o caracter
			li	tp, '0'				# TP = 48 = '0'
			blt 	t1, tp, erroreadFloat		# nao eh caractere valido para numero
			li	tp, '9'				# TP = 57 = '9'
			bgt 	t1, tp, erroreadFloat		# nao eh caractere valido para numero
			addi 	t1, t1, -48			# converte ascii para decimal
			fcvt.s.w  ft2, t1			# digito lido em float

			fmul.s 	ft2,ft2,ft3			# multiplica por un/dezena/centena
			fadd.s 	fa0,fa0,ft2			# soma no resultado
			fmul.s 	ft3,ft3,ft6			# proxima dezena/centena

			addi 	t0,t0,-1			# endereco anterior
			j 	loopintreadFloat		# volta ao loop
fimintreadFloat:

### Encontra a parte fracionaria  ja em fa0							
fracreadFloat:		fcvt.s.w 	ft2, zero		# zera parte fracionaria
			addi 	t0, s2, 1			# endereco depois do ponto
			fdiv.s 	ft3, ft1, ft6			# ft3 inicial 0.1
	
loopfracreadFloat: 	bge 	t0, s3, fimfracreadFloat	# endereco eh 'e' 'E' ou >ultimo
			lb 	t1, 0(t0)			# le o caracter
			li	tp, '0'				# TP = 48 = '0'
			blt 	t1, tp, erroreadFloat		# nao eh valido
			li	tp, '9'				# TP = 57 = '9'
			bgt 	t1, tp, erroreadFloat		# nao eh valido
			addi 	t1, t1, -48			# converte ascii para decimal
			fcvt.s.w 	ft2, t1			# digito lido em float		

			fmul.s 	ft2, ft2, ft3			# multiplica por ezena/centena
			fadd.s 	fa0, fa0, ft2			# soma no resultado
			fdiv.s 	ft3, ft3, ft6			# proxima frac un/dezena/centena
	
			addi 	t0, t0, 1			# proximo endereco
			j 	loopfracreadFloat		# volta ao loop		
fimfracreadFloat:

### Encontra a potencia em ft2

potreadFloat:		fcvt.s.w 	ft2, zero		# zera potencia
			addi 	t0, s3, 1			# endereco seguinte ao 'e'
			li 	s4, 0				# sinal do expoente positivo
			lb 	t1, 0(t0)			# le o caractere seguinte ao 'e'
			li	tp, '-'				# TP = 45 = '-'
			beq	t1, tp, potsinalnegreadFloat	# sinal do expoente esta escrito e eh positivo
			li	tp, '+'				# TP = 43 = '+'
			beq 	t1, tp, potsinalposreadFloat	# sinal do expoente eh negativo
			j 	pulapotsinalreadFloat		# nao esta escrito o sinal do expoente
potsinalnegreadFloat:	li 	s4, 1				# s4=1 expoente negativo
potsinalposreadFloat:	addi 	t0, t0, 1			# se tiver '-' ou '+' avanca para o proximo endereco
pulapotsinalreadFloat:	mv 	s5, t0 				# Neste ponto s5 contem o endereco do primeiro digito da pot e s4 o sinal do expoente		

			fmv.s 	ft3, ft1			# ft3 un/dez/cen = 1
	
### Encontra o expoente inteiro em t2
expreadFloat:		li 	t2, 0				# zera expoente
			mv 	t0, s0				# endereco do ultimo caractere da string
			li 	t3, 10				# numero dez
			li 	t4, 1				# und/dez/cent
				
loopexpreadFloat:	blt 	t0, s5, fimexpreadFloat		# ainda nao eh o endereco do primeiro digito?
			lb 	t1, 0(t0)			# le o caracter
			addi 	t1, t1, -48			# converte ascii para decimal
			mul 	t1, t1, t4			# mul digito
			add 	t2, t2, t1			# soma ao exp
			mul 	t4, t4, t3			# proxima casa decimal
			addi 	t0, t0, -1			# endereco anterior
			j loopexpreadFloat			# volta ao loop
fimexpreadFloat:
																																																								
# calcula o numero em ft2 o numero 10^exp
			fmv.s 	ft2, ft1			# numero 10^exp  inicial=1
			fmv.s 	ft3, ft6			# se o sinal for + ft3 eh 10
			li	tp, 0x00000000			# TP = ZERO
			beq 	s4, tp, sinalexpPosreadFloat	# se sinal exp positivo
			fdiv.s 	ft3, ft1, ft6			# se o final for - ft3 eh 0.1
sinalexpPosreadFloat:	li 	t0, 0				# contador 
sinalexpreadFloat: 	beq 	t0, t2, fimsinalexpreadFloat	# se chegou ao fim
			fmul.s 	ft2, ft2, ft3			# multiplica pelo fator 10 ou 0.1
			addi 	t0, t0, 1			# incrementa o contador
			j 	sinalexpreadFloat
fimsinalexpreadFloat:

		fmul.s 	fa0, fa0, ft2		# multiplicacao final!
	
		la 	t0, TempBuffer		# ajuste final do sinal do numero
		lb 	t1, 0(t0)		# le primeiro caractere
		li	tp, '-'			# TP = 45 = '-'
		bne 	t1, tp, fimreadFloat	# nao eh '-' entao fim
		fneg.s 	fa0, fa0		# nega o numero float

erroreadFloat:		
fimreadFloat: 	lw 	ra, 0(sp)		# recupera ra
		addi 	sp, sp, 4		# libera espaco
		ret				# retorna


############################################
#  Time                            	   #
#  a0    =    TimeLOW                 	   #
#  a1    =    TimeHIGH	                   #
############################################
Time:  	DE1(s8,Time.DE1)
	li 	a7, 30				# Chama o ecall do Rars
	ecall
	ret					# saida
	
Time.DE1:	csrr a0, time			#  Le time LOW
		csrr a1, timeh 			#  Le time HIGH
		ret
#Time.DE2: 	#li 	t0, TimerLOW		# endereco do Timer
#		li 	t0, STOPWATCH		# carrega endereco do StopWatch
#	 	lw 	a0, 0(t0)		# carrega o valor do contador de ms
#	 	#lw	a1, 4(t0)		# parte mais significativa
#	 	li 	a1, 0x0000		# contador eh de 32 bits
#fimTime: 	ret				# retorna


############################################
#  Sleep                            	   #
#  a0    =    Tempo em ms             	   #
############################################
Sleep:  DE1(s8,Sleep.DE1)
	li 	a7, 32				# Chama o ecall do Rars
	ecall
	ret					#Saida

Sleep.DE1:	csrr 	t0, time		# Le o tempo do sistema
		add 	t1, t0, a0		# soma com o tempo solicitado
Sleep.Loop:	csrr	t0, time		# Le o tempo do sistema
		blt	t0, t1, Sleep.Loop	# t0<t1 ?
		ret

#sleepDE2:	#li	t0,TimerLOW		# Endereco do TimerLOW
#		li 	t0, STOPWATCH		# endereco StopWatch
#		lw 	t1, 0(t0)		# carrega o contador de ms
#		add 	t2, a0, t1		# soma com o tempo solicitado pelo usuario
#		
#LoopSleep: 	lw 	t1, 0(t0)		# carrega o contador de ms
#		blt 	t1, t2, LoopSleep	# nao chegou ao fim volta ao loop
#	
#fimSleep: 	ret				# retorna


############################################
#  Random                            	   #
#  a0    =    numero randomico        	   #
############################################

Random:	 DE1(s8,Random.DE1)
	li 	a7,41			# Chama o ecall do Rars
	ecall	
	ret				# saida
	
Random.DE1: 	li 	t0, LFSR	# carrega endereco do LFSR
		lw 	a0, 0(t0)	# le a word em a0
		ret			# retorna


#################################
#    CLS                        #
#  Clear Screen                 #
#  a0 = cor      		#
#  a1 = frame                   #
#################################

clsCLS:	beq 	a1, zero, CLS.frame0
	li      t1, VGAADDRESSINI1              # Memoria VGA 1
   	li      t2, VGAADDRESSFIM1
   	j 	CLS.pula
CLS.frame0: 	li      t1, VGAADDRESSINI0           # Memoria VGA 0
   	    	li      t2, VGAADDRESSFIM0   	
CLS.pula:	andi    a0, a0, 0x00FF
#    	 li 	 t0, 0x01010101
#    	 mul	 a0, t0, a0
 		mv 	t0, a0
 		slli 	a0, a0, 8
 		or 	t0, t0, a0
 		slli 	a0, a0, 8
 		or 	t0, t0, a0
 		slli 	a0, a0, 8
 		or 	t0, t0, a0
 		
CLS.for:	beq     t1, t2, CLS.fim
		sw      t0, 0(t1)
    		addi    t1, t1, 4
    		j       CLS.for
CLS.fim:	ret


#########################################################################
#  Draw Line                  						#
#  Desenha uma linha do ponto (a0,a1) ao ponto (a2,a3) com a cor a4     #
# na Frame a5 (0 ou 1)							#
#########################################################################

BRESENHAM: 	li	a6, VGAADDRESSINI0           	# Memoria VGA 0
	   	beq	a5, zero, pulaBRES
	   	li 	a6, VGAADDRESSINI1              # Memoria VGA 1
	   	
pulaBRES: 	li 	a7, 320
	  	sub 	t0, a3, a1
	  	bge 	t0, zero, PULAABRES
	  	sub 	t0, zero, t0
PULAABRES:	sub 	t1, a2, a0
	   	bge  	t1, zero, PULABBRES
	   	sub  	t1, zero, t1	
PULABBRES: 	bge  	t0, t1, PULACBRES
	   	ble  	a0, a2, PULAC1BRES
	   	mv 	a5, a0
	   	mv 	a0, a2
	   	mv 	a2, a5
	   	mv	a5, a1
	   	mv 	a1, a3
	   	mv 	a3, a5
PULAC1BRES:	j PLOTLOWBRES

PULACBRES: 	ble  	a1, a3, PULAC2BRES
	   	mv 	a5, a0
	   	mv 	a0, a2
	   	mv 	a2, a5
	   	mv 	a5, a1
	   	mv 	a1, a3
	   	mv 	a3, a5
PULAC2BRES:	j PLOTHIGHBRES

PLOTLOWBRES:	sub 	t0, a2, a0		# dx=x1-x0
	 	sub 	t1, a3, a1		# dy y1-y0
	 	li  	t2, 1			# yi=1
	 	bge 	t1, zero, PULA1BRES	# dy>=0 PULA
	 	li  	t2, -1			# yi=-1
	 	sub 	t1, zero, t1		# dy=-dy
PULA1BRES:	slli 	t3, t1, 1		# 2*dy
		sub 	t3, t3, t0		# D=2*dy-dx
		mv 	t4, a1			# y=y0
		mv 	t5, a0			# x=x0
	
LOOPx1BRES:	TEM_M(s8,BRESENHAM.mul1)
		MULTIPLY(t6, t4, a7)
		j BRESENHAM.mul1d
BRESENHAM.mul1:	mul 	t6, t4, a7		# y*320
BRESENHAM.mul1d:add 	t6, t6, t5		# y*320+x
		add 	t6, t6, a6		# 0xFF000000+y*320+x
		sb 	a4, 0(t6)		# plot com cor a4
	
		ble 	t3, zero, PULA2BRES	# D<=0
		add 	t4, t4, t2		# y=y+yi
		slli 	t6, t0, 1		# 2*dx
		sub 	t3, t3, t6		# D=D-2dx
PULA2BRES:	slli 	t6, t1, 1		# 2*dy
		add 	t3, t3, t6		# D=D+2dx
		addi	t5, t5, 1
		bne 	t5, a2, LOOPx1BRES
		ret
		
PLOTHIGHBRES: 	sub 	t0, a2, a0		# dx=x1-x0
	 	sub 	t1, a3, a1		# dy y1-y0
	 	li 	t2, 1			# xi=1
	 	bge 	t0, zero, PULA3BRES	# dy>=0 PULA
	 	li 	t2, -1			# xi=-1
	 	sub 	t0, zero, t0		# dx=-dx
PULA3BRES:	slli 	t3, t0, 1		# 2*dx
		sub 	t3, t3, t1		# D=2*dx-d1
		mv 	t4, a0			# x=x0
		mv 	t5, a1			# y=y0
	
LOOPx2BRES:	TEM_M(s8,BRESENHAM.mul2)
		MULTIPLY(t6, t5, a7)
		j BRESENHAM.mul2d
BRESENHAM.mul2:	mul 	t6, t5, a7		# y*320
BRESENHAM.mul2d:add 	t6, t6, t4		# y*320+x
		add 	t6, t6, a6		# 0xFF000000+y*320+x
		sb 	a4, 0(t6)		# plot com cor a4
	
		ble 	t3, zero, PULA4BRES	# D<=0
		add 	t4, t4, t2		# x=x+xi
		slli 	t6, t1, 1		# 2*dy
		sub 	t3, t3, t6		# D=D-2dy
PULA4BRES: 	slli 	t6, t0, 1		# 2*dy
		add 	t3, t3, t6		# D=D+2dx
		addi 	t5, t5, 1
		bne 	t5, a3, LOOPx2BRES
		ret		


# Sugestao para nomes de loops: Sempre comecar com o nome da sub-rotina, então adicionar um '.', seguido do nome do loop . Garante que o nome do loop será único, se as sub-rotinas
# tiverem nomes diferentes.

#############################################
#  PrintIntUnsigned                         #
#  a0    =    valor inteiro                 #
#  a1    =    x                             #
#  a2    =    y  			    #
#  a3    =    cor                           #
#  a4    =    frame			    #
#############################################

printIntUnsigned:	addi 	sp, sp, -4		# Aloca espaco
		sw 	ra, 0(sp)			# salva ra
		la 	t0, TempBuffer			# carrega o Endereco do Buffer da String
		
		li 	t2, 10				# carrega numero 10
		li 	t1, 0				# carrega numero de digitos com 0

printIntUnsigned.loop1:	TEM_M(s8,printIntUnsigned.pula1)
			DIVU10(t4,a0)
			REMU10(t3,a0)
			j	printIntUnsigned.pula1d
printIntUnsigned.pula1:	divu 	t4, a0, t2			# divide por 10 (quociente)
			remu 	t3, a0, t2			# resto
printIntUnsigned.pula1d:addi 	sp, sp, -4			# aloca espaco na pilha
		sw 	t3, 0(sp)			# coloca resto na pilha
		mv 	a0, t4				# atualiza o numero com o quociente
		addi 	t1, t1, 1			# incrementa o contador de digitos
		bne 	a0, zero, printIntUnsigned.loop1# verifica se o numero eh zero
				
printIntUnsigned.loop2:	lw 	t2, 0(sp)		# le digito da pilha
		addi 	sp, sp, 4			# libera espaco
		addi 	t2, t2, 48			# converte o digito para ascii
		sb 	t2, 0(t0)			# coloca caractere no buffer
		addi 	t0, t0, 1			# incrementa endereco do buffer
		addi 	t1, t1, -1			# decrementa contador de digitos
		bne 	t1, zero, printIntUnsigned.loop2# eh o ultimo?
		sb 	zero, 0(t0)			# insere \NULL na string
		
		la 	a0, TempBuffer			# Endereco do buffer da srting
		jal 	printString			# chama o print string
				
		lw 	ra, 0(sp)			# recupera a
		addi 	sp, sp, 4			# libera espaco
printIntUnsigned.fim:	ret






###########################################################################
# lib de operações multiplicação, divisão e resto para a ISA RV32I
# Nomenclatura usada pelo gcc

# Multiplicação signed em a0 e a1  retorno em a0
# https://github.com/gcc-mirror/gcc/tree/master/libgcc/config/epiphany
__mulsi3:	addi 	sp,sp,-12
		sw 	a1,0(sp)
		sw 	a4,4(sp)
		sw	a5,8(sp)
	
	 	mv      a5,a0
        	li      a0,0
mulsi3.L4: 	beqz    a5,mulsi3.L1
        	andi    a4,a5,1
        	beqz    a4,mulsi3.L3
        	add     a0,a0,a1
mulsi3.L3: 	srli    a5,a5,1
        	slli    a1,a1,1
        	j       mulsi3.L4
        	
mulsi3.L1: 	lw 	a1,0(sp)
		lw	a4,4(sp)
		lw	a5,8(sp)
		addi 	sp,sp,12
		ret
        
# Divisão unsigned em a0 e a1 retorno em a0
# https://stackoverflow.com/questions/34457575/integer-division-algorithm-analysis
__udivsi3:	addi 	sp,sp,-16
		sw 	a1,0(sp)
		sw	a3,4(sp)
		sw 	a4,8(sp)
		sw	a5,12(sp)
		
 		mv      a4,a0
        	srli    a3,a0,1
        	li      a5,1
udivsi3.L3:    	bltu    a3,a1,udivsi3.L6
        	slli    a5,a5,1
        	slli    a1,a1,1
        	j       udivsi3.L3
udivsi3.L6:    	li      a0,0
udivsi3.L2:   	beqz    a5,udivsi3.L1
        	bltu    a4,a1,udivsi3.L5
        	sub     a4,a4,a1
        	add     a0,a0,a5
udivsi3.L5:    	srli    a5,a5,1
        	srli    a1,a1,1
        	j       udivsi3.L2
        	
udivsi3.L1: 	lw 	a1,0(sp)
		lw	a3,4(sp)
		lw	a4,8(sp)
		lw	a5,12(sp)
		addi 	sp,sp,16
    		ret

# Resto unsigned em a0 e a1
__umodsi3:	addi	sp, sp, -12
		sw 	t0, 0(sp)
		sw 	t1, 4(sp)
		sw 	ra, 8(sp)
	 	mv 	t0, a0		# dividendo
		mv 	t1, a1		# divisor
		jal 	__udivsi3
		mv 	a1, t1		# quociente * divisor
		jal 	__mulsi3
		sub 	a0, t0, a0	# dividendo-quociente*divisor
		lw 	t0, 0(sp)
		lw 	t1, 4(sp)
		lw 	ra, 8(sp)
		addi 	sp, sp, 12
		ret
		
# Divisão signed em a0 e a1	
__divsi3:	addi	sp, sp, -16
		sw 	t0, 0(sp)
		sw 	t1, 4(sp)
		sw 	t2, 8(sp)
		sw 	ra, 12(sp)
		srai	t0,a0,31	# indica se a0 é pos(0) ou neg (2^32-1)
		srai 	t1,a1,31	# indica se a1 é pos(0) ou neg (2^32-1)
		xor	t2,t0,t1	# indica se deve(!=0) ou não(==0) inverter o sinal do resultado
		beqz 	t0,divsi3.pula1
		neg	a0,a0		# nega
divsi3.pula1:	beqz 	t1,divsi3.pula2
		neg	a1,a1		# nega
divsi3.pula2:	jal 	__udivsi3	# divisão unsigned
		beqz	t2, divsi3.pula3	
		neg	a0,a0		# nega
divsi3.pula3:	lw 	t0, 0(sp)
		lw 	t1, 4(sp)
		lw 	t2, 8(sp)
		lw 	ra, 12(sp)
		addi 	sp, sp, 16
		ret
						
# Resto signed em a0 e a1	
__modsi3:	addi	sp, sp, -12
		sw 	t0, 0(sp)
		sw 	t1, 4(sp)
		sw 	ra, 8(sp)
		srai	t0,a0,31	# indica se a0 é pos(0) ou neg (2^32-1)
		srai 	t1,a1,31	# indica se a1 é pos(0) ou neg (2^32-1)
		beqz 	t0,modsi3.pula1
		neg	a0,a0		# nega
modsi3.pula1:	beqz 	t1,modsi3.pula2
		neg	a1,a1		# nega
modsi3.pula2:	jal 	__umodsi3	# resto unsigned
		beqz	t0, modsi3.pula3	# sinal do dividendo	
		neg	a0,a0		# nega
modsi3.pula3:	lw 	t0, 0(sp)
		lw 	t1, 4(sp)
		lw 	ra, 8(sp)
		addi 	sp, sp, 12
		ret																				
		
