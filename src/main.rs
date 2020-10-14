mod renderer;
mod simulator;

use std::error::Error;
use std::thread;

fn main() -> Result<(), Box<dyn Error>> {
    let mut sim = simulator::Simulator::new().load_from_file("something.s")?;

    for instruction in sim.code.iter() {
        println!("{:?}", instruction);
    }

    let mmio = sim.memory.mmio.clone();
    thread::spawn(move || {
        sim.run();
    });

    renderer::init(mmio);

    Ok(())
}
