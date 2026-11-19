mod devices;
mod settings;

mod modules;
mod keyboard;
mod synthesizer;
mod sound;
mod notes;

use std::collections::HashMap;
use devices::Device;
use modules::*;
use crate::keyboard::Keyboard;
use crate::synthesizer::Synthesizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    env_logger::init();

    let device = Device::new()?;
    let mut s = Synthesizer::new(device);

    // create LFO
    let lfo = LFO::new("LFO1");
    s.add_module(Box::new(lfo));
    // create oscillator
    let oscillator = Oscillator::new("Wave1");
    s.add_module(Box::new(oscillator));

    // LFO takes no input and writes to buffer 0
    s.add_connection("LFO1", HashMap::new(), 0);
    // Oscillator takes frequency input from buffer 0 and writes to buffer 1
    s.add_connection("Wave1", HashMap::from([(Input::Frequency, 0)]), 1);

    // --- configure keyboard
    let mut keyboard = Keyboard::new(s.sender());
    keyboard.connect_frequency("Wave1");
    keyboard.connect_gate("Wave1");
    std::thread::spawn(move || keyboard.run());     // Spawn a separate thread to run the keyboard input loop.

    // s.send_message(RtMessage::new("LFO1", Parameter::Gate(true)));
    s.send_message(RtMessage::new("LFO1", Parameter::Depth(0.01)));
    println!("Synthesizer running. Press Ctrl+C to stop.");
    s.run()?;

    return Ok(());
}