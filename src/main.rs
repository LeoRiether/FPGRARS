mod renderer;
mod simulator;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut sim = simulator::Simulator::new()
        .load_from_file("something.s")?;

    let mut memory = simulator::Memory::new();
    renderer::init(memory.mmio.clone());

    Ok(())
}
