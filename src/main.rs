use owo_colors::OwoColorize;
use fpgrars::simulator::Simulator;
use std::error::Error;
use std::thread;
use fpgrars::config::CONFIG;

fn main() -> Result<(), Box<dyn Error>> {
    let memory = fpgrars::simulator::memory::Memory::new();
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
        let state = fpgrars::renderer::State::new(mmio, CONFIG.width, CONFIG.height, CONFIG.scale);
        fpgrars::renderer::init(state);
    }

    sim_thread.join().unwrap();
    Ok(())
}
