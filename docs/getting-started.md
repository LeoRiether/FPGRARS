# Getting Started
## With FPGRARS and RISC-V assembly

In this guide, we'll go over the basics of running RISC-V assembly programs
with FPGRARS!

## Installation
If you haven't yet downloaded FPGRARS, head over to the [latest release](https://github.com/LeoRiether/FPGRARS/releases/latest)
and download the appropriate executable. The easiest way to "install" it is by putting it in the same
folder as our assembly file:

```
my-wonderful-riscv-project
|- fpgrars-executable
|- riscv.s
```

If you later want to use FPGRARS for multiple projects, it's best to put the
executable in a folder that's in your `PATH` environment variable (could be
`~/.local/bin` for Unix, idk where for Windows sorry /shrug).

If you have the Rust toolchain installed, you can also download FPGRARS by
running `cargo install fpgrars`.

## Hello World!

Create a `riscv.s` file and put the following content in it:

```assembly
.data
# Define the label "hello" to as the address to the very start
# of the "Hello World!\n" string 
hello: .string "Hello World!\n"

.text
    li a7, 4     # Load 4 (the ecall for "print string") into the a7 register
    la a0, hello # Load the address of the label 'hello' into a0
    ecall        # Perform an environment call

    li a7, 11    # Load 11 ("print character" ecall) into a7
    li a0, '\n'  # Load '\n' into a0
    ecall        # Perform an environment call

    li a7, 10    # "exit" ecall
    li a0, 0     # exit with code 0 
    ecall
```

You can now run the program with the command `./fpgrars riscv.s` and you should see a "Hello World"
appear on your terminal!

Note that `./fpgrars` should have the full name of the executable file. On Windows, it's probably
something like `./fpgrars-x86_64-pc-windows-msvc--original.exe`, on Linux,
`./fpgrars-x86_64-unknown-linux-gnu--original `. Of course, you can rename it
to whatever you want, like just `fpgrars`.

## Using the Bitmap Display

This section is WIP :) 

While I'm working on that, check out the [samples folder](https://github.com/LeoRiether/FPGRARS/tree/main/samples)!
There are many examples there that may help you. I guess [keyboard_and_display_demo.s](https://github.com/LeoRiether/FPGRARS/blob/main/samples/keyboard_and_display_demo.s)
is the easiest to follow, but [polygon.s](https://github.com/LeoRiether/FPGRARS/blob/main/samples/polygon.s)
is definitely the coolest.

