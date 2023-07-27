# FPGRARS
## Fast Pretty Good RISC-V Assembly Rendering System

FPGRARS is a RISC-V assembly simulator with a graphics display window and keyboard input, similar to [RARS](https://github.com/TheThirdOne/rars), but 200 times faster. If you want to run RISC-V assembly programs easily, learn assembly language, or even build a game, FPGRARS is a great option! (and if it isn't for you please open an [issue)](https://github.com/LeoRiether/FPGRARS/issues) :) 

## Running FPGRARS
First, head over to the [latest release](https://github.com/LeoRiether/FPGRARS/releases/latest) and download the appropriate executable. Then, you can run a RISC-V assembly file either by running `./fpgrars your_riscv_file.s` in a terminal or by dragging the `.s` onto the executable. If you're on Linux, you might need to `chmod +x fpgrars-x86_64-unknown-linux-gnu` for FPGRARS to work.

If you have the Rust toolchain installed, you can also download FPGRARS by running `cargo install fpgrars`.

You may also want to check out [Getting Started](https://leoriether.github.io/FPGRARS/getting-started/) for a more detailed guide.
