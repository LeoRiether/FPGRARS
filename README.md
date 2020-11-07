# FPGRARS
## Fast Pretty Good RISC-V Assembly Rendering System
###### (name may change at any moment)

[![build status](https://github.com/LeoRiether/FPGRARS/workflows/Build%20&%20Test/badge.svg)](https://github.com/LeoRiether/FPGRARS/actions)

FPGRARS aims to provide a RISC-V assembly simulator with 8-bit color graphics display and keyboard input, similar to [RARS](https://github.com/TheThirdOne/rars), but faster.

## Supported ecalls

| Description | a7 | Input | Output |
|-------------|----|-------|--------|
Print integer | 1  | a0 = integer to print | |
Print string | 4 | a0 = address of the string | |
Read int | 5 | | a0 = the read integer |
Print float | 6 | a0 = float to print | |
Exit | 10 | | |
Time | 30 | | a0 = low bits of milliseconds since unix epoch, a1 = high bits |
Midi out | 31 | does nothing for now | |
Sleep ms | 32 | a0 = number of milliseconds to sleep | |
Midi out sync | 33 | does nothing for now |
Rand seed | 40 | does nothing for now | |
Rand int | 41 | | a0 = random integer |
Rand int range | 42 | a0 is discarded, a1 = upper bound | a0 = random integer in [0, a1) |
Rand float | 43 | | fa0 = random float in [0, 1) |
Clear screen | 48 or 148 | a0 = color, a1 = frame | |
