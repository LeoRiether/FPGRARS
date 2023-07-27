# FPGRARS
## Fast Pretty Good RISC-V Assembly Rendering System
###### (name may change at any moment)

[![build status](https://github.com/LeoRiether/FPGRARS/workflows/Build%20&%20Test/badge.svg)](https://github.com/LeoRiether/FPGRARS/actions)

FPGRARS is a RISC-V assembly simulator with a graphics display window and keyboard input, similar to [RARS](https://github.com/TheThirdOne/rars), but 200 times faster. If you want to run RISC-V assembly programs easily, learn assembly language, or even build a game, FPGRARS is a great option! (and if it isn't for you please open an [issue](https://github.com/LeoRiether/FPGRARS/issues) :) 

## Running FPGRARS
First, head over to the [latest release](https://github.com/LeoRiether/FPGRARS/releases/latest) and download the appropriate executable. Then, you can run a RISC-V assembly file either by running `./fpgrars your_riscv_file.s` in a terminal or by dragging the `.s` onto the executable. If you're on Linux, you might need to `chmod +x fpgrars-x86_64-unknown-linux-gnu` for FPGRARS to work.

If you have the Rust toolchain installed, you can also download FPGRARS by running `cargo install fpgrars`.

You may also want to check out [Getting Started](https://leoriether.github.io/FPGRARS/getting-started/) for a more detailed guide.

## Documentation
Still in construction at https://leoriether.github.io/FPGRARS/

## Supported ecalls

| Description | a7 | Input | Output |
|-------------|----|-------|--------|
Print integer | 1  | a0 = integer to print | |
Print float | 2 | fa0 = float to print | |
Print string | 4 | a0 = address of the string | |
Read int | 5 | | a0 = the read integer |
Read float | 6 | | fa0 = the read float |
Sbrk | 9 | a0 = bytes to allocate (>= 0) | a0 = address of the allocated chunk
Exit | 10 | | |
Stop execution | 110 | |
Print char | 11 | a0 = the char | |
Time | 30 | | a0 = low bits of milliseconds since unix epoch, a1 = high bits |
Midi out | 31 | a0 = pitch (note), a1 = duration in ms, a2 = instrument (in range 0-127), a3 = volume (also 0-127) | Async sound |
Sleep ms | 32 | a0 = number of milliseconds to sleep | |
Midi out sync | 33 | a0 = pitch (note), a1 = duration in ms, a2 = instrument (in range 0-127), a3 = volume (also 0-127) | Synchronous sound |
Print hex integer | 34 | a0 = integer to print | |
Print unsigned integer | 36 | a0 = unsigned integer to print | |
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
