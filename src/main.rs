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
pub mod utf8_lossy_lines;
pub mod util;

use lazy_static::lazy_static;
use owo_colors::OwoColorize;
use simulator::Simulator;
use std::error::Error;
use std::thread;

lazy_static! {
    pub static ref ARGS: args::Args = args::get_args();
}

fn main() -> Result<(), Box<dyn Error>> {
    let memory = simulator::memory::Memory::new();
    let mmio = memory.mmio.clone();

    let sim_thread = thread::Builder::new()
        .name("FPGRARS Simulator".into())
        .spawn(move || {
            let mut sim = Simulator::default()
                .with_memory(memory)
                .with_midi_port(ARGS.port);

            if let Err(e) = sim.load_file(&ARGS.file) {
                eprintln!("   {}: {}\n", "[error]".bright_red().bold(), e);
                std::process::exit(1);
            };

            let start_time = std::time::Instant::now();
            sim.run();
            eprintln!("Finished in {}ms", start_time.elapsed().as_millis());
            std::process::exit(0);
        })?;

    if !ARGS.no_video {
        renderer::init(mmio);
    }

    sim_thread.join().unwrap();
    Ok(())
}
