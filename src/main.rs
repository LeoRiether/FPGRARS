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

mod config;
mod instruction;
mod parser;
mod renderer;
mod simulator;
pub mod utf8_lossy_lines;
pub mod util;

use owo_colors::OwoColorize;
use simulator::Simulator;
use std::error::Error;
use std::thread;

use crate::config::CONFIG;

fn main() -> Result<(), Box<dyn Error>> {
    let memory = simulator::memory::Memory::new();
    let mmio = memory.mmio.clone();

    let sim_thread = thread::Builder::new()
        .name("FPGRARS Simulator".into())
        .spawn(move || {
            let mut sim = Simulator::default()
                .with_memory(memory)
                .with_midi_port(CONFIG.port);

            if let Err(e) = sim.load_file(&CONFIG.file) {
                eprintln!("   {}: {}\n", "[error]".bright_red().bold(), e);
                std::process::exit(1);
            };

            let start_time = std::time::Instant::now();
            let exit_code = sim.run();
            eprintln!("Finished in {}ms", start_time.elapsed().as_millis());
            std::process::exit(exit_code);
        })?;

    if !CONFIG.no_video {
        let state = renderer::State::new(mmio, CONFIG.width, CONFIG.height, CONFIG.scale);
        renderer::init(state);
    }

    sim_thread.join().unwrap();
    Ok(())
}
