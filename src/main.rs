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

mod args;
mod instruction;
mod parser;
mod renderer;
mod simulator;

use std::error::Error;
use std::thread;

use owo_colors::OwoColorize;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = args::get_args();
    let file = std::mem::take(&mut args.file);

    let sim = simulator::Simulator::new(args.port);
    let mmio = sim.memory.mmio.clone();

    let sim_thread = thread::Builder::new()
        .name("FPGRARS Simulator".into())
        .spawn(move || {
            let mut sim = match sim.load_from_file(&file) {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("   {}: {}\n", "[error]".bright_red().bold(), e);
                    std::process::exit(1);
                }
            };

            let start_time = std::time::Instant::now();
            sim.run();
            eprintln!("Finished in {}ms", start_time.elapsed().as_millis());
            std::process::exit(0);
        })?;

    if !args.no_video {
        renderer::init(mmio);
    }

    sim_thread.join().unwrap();
    Ok(())
}
