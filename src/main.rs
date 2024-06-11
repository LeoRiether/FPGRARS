use fpgrars::simulator::Simulator;
use owo_colors::OwoColorize;
use std::error::Error;
use std::thread;

fn main() -> Result<(), Box<dyn Error>> {
    let config = fpgrars::config::Config::get();

    let memory = fpgrars::simulator::memory::Memory::new();
    let mmio = memory.mmio.clone();

    let sim_thread = thread::Builder::new()
        .name("FPGRARS Simulator".into())
        .spawn(move || {
            let mut sim = Simulator::default()
                .with_memory(memory)
                .with_midi_port(config.port);

            if let Err(e) = sim.load_file(&config.file) {
                eprintln!("   {}: {}\n", "[error]".bright_red().bold(), e);
                std::process::exit(1);
            };

            let start_time = std::time::Instant::now();
            let exit_code = sim.run();
            eprintln!("Finished in {}ms", start_time.elapsed().as_millis());
            std::process::exit(exit_code);
        })?;

    if !config.no_video {
        let state = fpgrars::renderer::State::new(mmio, config.width, config.height, config.scale);
        fpgrars::renderer::init(state);
    }

    sim_thread.join().unwrap();
    Ok(())
}
