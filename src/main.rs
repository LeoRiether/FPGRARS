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

extern crate clap;

mod app;
mod parser;
mod renderer;
mod simulator;

use std::env;
use std::error::Error;
use std::path::Path;
use std::thread;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = app::build_app().get_matches_from(env::args_os());

    if let Some(file) = matches.value_of("INPUT_FILE") {
        if !Path::new(file).is_file() {
            println!("\n`{}` must be a valid file.\n", file);
            std::process::exit(1);
        }
    }

    let sim = simulator::Simulator::new();
    let mmio = sim.memory.mmio.clone();

    let file = matches
        .value_of("INPUT_FILE")
        .expect("Failed to get <INPUT_FILE>")
        .to_string();

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
