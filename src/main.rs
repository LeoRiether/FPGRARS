//!
//! FPGRARS was made as an alternative to [RARS](https://github.com/TheThirdOne/rars), as it was
//! too slow for some applications. As such, it implements parsing and simulating RISC-V code,
//! as well as showing images on the screen and interacting with user input.
//!
//! Right now I don't aim to implement the instructions too close to what a real RISC-V processor
//! would execute. For example, there are some pseudoinstructions implemented as real instructions,
//! it's impossible to make self-modifying code and there's no difference between `jal` and `call`.
//! Even then, I think these won't make too much of a difference for most users.
//!
//! Also note that the simulator cares less about correctness than RARS, so some programs that run
//! here will fail there. One such case occurs if you read a word from an unaligned position in memory,
//! FPGRARS doesn't care, but RARS complains.
//!

mod renderer;
mod simulator;
mod parser;

use std::env;
use std::error::Error;
use std::thread;

fn main() -> Result<(), Box<dyn Error>> {
    let sim = simulator::Simulator::new();
    let mmio = sim.memory.mmio.clone();

    let mut args: Vec<String> = env::args().skip(1).collect();
    let file = args.pop().expect("Usage: ./fpgrars [OPTIONS] riscv_file.s");

    thread::Builder::new()
        .name("FPGRARS Simulator".into())
        .spawn(move || {
            let mut sim = match sim.load_from_file(file) {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("An error occurred while parsing your code:\n{:?}", e);
                    std::process::exit(0);
                }
            };

            let start_time = std::time::Instant::now();
            sim.run();
            println!("Finished in {}ms", start_time.elapsed().as_millis());
            std::process::exit(0);
        })?;

    renderer::init(mmio);

    Ok(())
}
