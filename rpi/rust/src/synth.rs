mod devices;
mod settings;

mod modules;
mod keyboard;
mod synthesizer;
mod sound;
mod notes;
mod envelope;

use std::collections::HashMap;
use devices::Device;
use modules::*;
use crate::envelope::ADSREnvelope;
use crate::keyboard::Keyboard;
use crate::synthesizer::Synthesizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    env_logger::init();

    let device = Device::new()?;
    let mut s = Synthesizer::new(device);

    // ---------- create LFO
    let lfo = LFO::new("LFO1");
    s.add_module(Box::new(lfo));
    // ---------- create oscillator
    let oscillator = Oscillator::new("Wave1");
    s.add_module(Box::new(oscillator));
    // ---------- create envelope
    let envelope = ADSREnvelope::new("Env1",
        0.05, 0.35, 0.5, 1.0, 0.3, 0.1);
    s.add_module(Box::new(envelope));

    // ---------- LFO takes no input and writes to buffer 0
    s.add_connection("LFO1", HashMap::new(), 0);
    // ---------- Oscillator takes frequency input from buffer 0 and writes to buffer 1
    s.add_connection("Wave1", HashMap::from([(Input::Frequency, 0)]), 1);
    // ---------- Oscillator takes audio input from buffer 1 and writes to buffer 2
    s.add_connection("Env1", HashMap::from([(Input::AudioIn, 1)]), 2);

    // ---------- configure keyboard
    let mut keyboard = Keyboard::new(s.sender());
    keyboard.connect_frequency("Wave1");
    keyboard.connect_gate("Env1");
    std::thread::spawn(move || keyboard.run());     // Spawn a separate thread to run the keyboard input loop.

    s.send_message(RtMessage::new("Wave1", Parameter::Gate(true)));
    // s.send_message(RtMessage::new("LFO1", Parameter::Gate(true)));
    s.send_message(RtMessage::new("LFO1", Parameter::Depth(0.02)));
    println!("Synthesizer running. Press Ctrl+C to stop.");
    s.run()?;

    return Ok(());
}