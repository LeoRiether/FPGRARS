# Configuration

There are two ways to configure FPGRARS: by passing command-line arguments or
by using an `fpgrars.toml` file.

## Command-line arguments

When you run fpgrars --help, you will see the available arguments for your specific version of FPGRARS, like it's shown below. Do note that the version on this website might not always be up to date. 

```
> fpgrars --help
Fast Pretty Good RISC-V Assembly Rendering System

Usage: fpgrars [OPTIONS] [FILE]

Arguments:
  [FILE]  The RISC-V file to execute

Options:
      --no-video            Hides the bitmap display
  -w, --width <WIDTH>       The width of the bitmap display. Defaults to 320px
  -h, --height <HEIGHT>     The height of the bitmap display. Defaults to 240px
  -s, --scale <SCALE>       Each pixel is scaled by this factor. Defaults to 2 (each pixel becomes a 2x2 square)
  -p, --port <PORT>         The MIDI port to use for audio
      --print-instructions  Prints the instructions in the FPGRARS format
      --print-state         Prints the final state of the program after execution
  -h, --help                Print help
  -V, --version             Print version
```

For example, if you want to run FPGRARS without the bitmap display and print
the state of the registers when the program exits, you can use the command 

```bash
fpgrars --no-video --print-state file.s
```

As another example, if you want the bitmap display to be 1280 x 720 and display
each pixel as a 1x1 square (an actual pixel), you should run

```bash
fpgrars -w 1280 -h 720 -s 1
```

In this case, you probably want to run FPGRARS with this configuration every
time your project is executed. The next section outlines a way to do this.

## Configuration file (fpgrars.toml)

If FPGRARS detects an `fpgrars.toml` in the current working directory, it will
use the configuration defined there by default. For example, suppose the
working directory is structured like this:

```
|- fpgrars.toml
|- src
|  |- main.s
```

and `fpgrars.toml` contains the following:

```toml title="fpgrars.toml"
width = 1280
height = 720
scale = 1
```

Then, running `fpgrars src/main.s` is the same as `fpgrars --width 1280 --height 720 --scale 1`.

You could also specify a default file to run, for example:

```toml title="fpgrars.toml"
file = "src/main.s"
```

Then, running `fpgrars` is the same as `fpgrars src/main.s`.

It's worth noting that you can still pass command-line arguments in addition to
using `fpgrars.toml`, in which case any options you pass by command-line will
take priority over the ones defined in the config file.

