mod devices;
mod oscillator;
mod settings;

mod module;
mod synthetizer;
mod keyboard;

use devices::Device;
use module::*;
use crate::synthetizer::Synthesizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    env_logger::init();

    let device = Device::new()?;
    let mut s = Synthesizer::new(device);

    println!("Synthesizer running. Press Ctrl+C to stop.");
    s.run()?;

    return Ok(());
}