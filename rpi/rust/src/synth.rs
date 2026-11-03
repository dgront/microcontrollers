mod devices;
mod oscillator;
mod settings;

use devices::Device;
use oscillator::Oscillator;
use settings::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut device = Device::new()?;
    let mut oscillator = Oscillator::new(DEFAULT_FREQUENCY);

    let mut buffer = vec![0.0_f32; BUFFER_FRAMES * CHANNELS];

    println!("Synthesizer running. Press Ctrl+C to stop.");

    loop {
        oscillator.fill_buffer(&mut buffer);
        device.write(&buffer)?;
    }
}