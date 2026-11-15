mod devices;
mod settings;

mod modules;
mod synthetizer;
mod keyboard;

use devices::Device;
use modules::*;
use crate::synthetizer::Synthesizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    env_logger::init();

    let device = Device::new()?;
    let mut s = Synthesizer::new(device);
    // create oscillator

    // register it with the synthesizer
    // create LFO, register it with the synthesizer
    // connect the two

    println!("Synthesizer running. Press Ctrl+C to stop.");
    s.run()?;

    return Ok(());
}