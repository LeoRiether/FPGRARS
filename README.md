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
Print char | 11 | a0 = the char | |
Exit | 10 | | |
Stop execution | 110 | |
Time | 30 | | a0 = low bits of milliseconds since unix epoch, a1 = high bits |
Midi out | 31 | does nothing for now | |
Sleep ms | 32 | a0 = number of milliseconds to sleep | |
Midi out sync | 33 | does nothing for now |
Rand seed | 40 | does nothing for now | |
Rand int | 41 | | a0 = random integer |
Rand int range | 42 | a0 is discarded, a1 = upper bound | a0 = random integer in [0, a1) |
Rand float | 43 | | fa0 = random float in [0, 1) |
Clear screen | 48 or 148 | a0 = color, a1 = frame | |
Open file | 1024 | a0 = address of the null-terminated string for the path, a1 = 0 (read mode), 1 (write mode) or 9 (append mode) | a0 = the file descriptor or -1 if error |
Close file | 57 | a0 = a file descriptor | |
Seek | 62 | a0 = a file descriptor, a1 = the offset to seek, a2 = 0 (seek from the start of the file), 1 (from the current position) or 2 (from the end) | a0 = the selected position from the start of the file |
Read | 63 | a0 = a file descriptor, a1 = address of the buffer, a2 = maximum length to read | a0 = number of bytes read or -1 if error |
Write | 64 | a0 = a file descriptor, a1 = address of the buffer, a2 = length to write | a0 = number of bytes written of -1 if error |
