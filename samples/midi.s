.data

.text

.macro play(%pitch, %dur)
	li a7 33 # MIDI Out Sync
	li a0 %pitch # Pitch
	li a1 %dur # Duration
	li a2 150
	mul a1 a1 a2 # Duration times 150
	li a2 29 # Instrument
	li a3 110 # Volume/Velocity
	ecall
.end_macro

	play(66, 4)
	play(65, 3)
	play(63, 1)
	play(61, 6)
	play(59, 2)
	play(58, 4)
	play(56, 4)
	play(54, 4)
