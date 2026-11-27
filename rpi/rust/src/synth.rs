mod devices;
mod settings;

mod modules;
mod keyboard;
mod synthesizer;
mod sound;
mod notes;
mod sequencer;

use std::collections::HashMap;
use devices::Device;
use modules::*;
use crate::keyboard::Keyboard;
use crate::notes::Note;
use crate::sequencer::Sequencer;
use crate::synthesizer::Synthesizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    env_logger::init();

    let device = Device::new()?;
    let mut s = Synthesizer::new(device);

    // ---------- create LFO
    let lfo = LFO::new("LFO1");
    s.add_module(Box::new(lfo));

    // ---------- create oscillator
    // let oscillator = Oscillator::new("Wave1");
    let oscillator = UnisonOscillator::new("Wave1", 5, 0.5);
    s.add_module(Box::new(oscillator));

    // ---------- create envelope
    let envelope = ADSREnvelope::new("Env1",
        0.05, 0.85, 0.3, 1.0, 0.3, 0.1);
    s.add_module(Box::new(envelope));

    // ---------- create chorus effect
    // let chorus = Chorus::new("Chorus");
    // s.add_module(Box::new(chorus));

    let filter = LowPassFilter::new("Filter1", 600.0);
    // let filter = BiquadFilter::new("Filter1", 660.0, 0.0);
    s.add_module(Box::new(filter));

    // ---------- create amplifier
    let amplifier = VCA::new("VCA");
    s.add_module(Box::new(amplifier));

    // ---------- LFO takes no input and writes to buffer 0
    s.add_connection("LFO1", HashMap::new(), 0);
    // ---------- Envelope writes to buffer 1
    s.add_connection("Env1", HashMap::new(), 1);

    // ---------- Oscillator takes LFO frequency input from buffer 0 and writes to buffer 2
    s.add_connection("Wave1", HashMap::from([(Input::Frequency, 0)]), 2);

    // ---------- Chorus takes audio input from buffer 2 and writes to buffer 3
    // s.add_connection("Chorus", HashMap::from([(Input::AudioIn, 2)]), 3);

    // ---------- Filter takes audio input from buffer 2 and envelope from buffer 1 and writes to buffer 3
    s.add_connection("Filter1", HashMap::from([(Input::AudioIn, 2),(Input::Frequency, 1)]), 3);
    s.add_connection("VCA", HashMap::from([(Input::AudioIn, 3),(Input::Gain, 1)]), 4);

    // ---------- configure keyboard
    let mut keyboard = Keyboard::new(s.sender());
    keyboard.connect_frequency("Wave1");
    keyboard.connect_gate("Env1");
    std::thread::spawn(move || keyboard.run());     // Spawn a separate thread to run the keyboard input loop.

    // ---------- configure sequencer
    let mut sequencer = Sequencer::new(s.sender());
    sequencer.set_steps(vec![
        (Note::C.octave_frequency(4), 0.8), // C4
        (Note::D.octave_frequency(4), 1.0), // D4
        (Note::E.octave_frequency(4), 1.0), // E4
        (Note::G.octave_frequency(4), 0.5), // G4
        (0.0,    0.0), // Rest
        (Note::G.octave_frequency(4), 0.8), // G4
        (Note::A.octave_frequency(4), 0.8), // A4
        (Note::C.octave_frequency(5), 1.0), // C5
    ]);
    sequencer.set_bpm(30.0);
    sequencer.connect_frequency("Filter1");
    sequencer.connect_frequency("Wave1");
    sequencer.connect_gate("Env1");
    std::thread::spawn(move || sequencer.run());     // Spawn a separate thread to run the keyboard input loop.


    s.send_message(RtMessage::new("Wave1", Parameter::Gate(true)));
    s.send_message(RtMessage::new("LFO1", Parameter::Depth(0.01)));
    s.send_message(RtMessage::new("LFO1", Parameter::Frequency(10.0)));
    // s.send_message(RtMessage::new("LFO1", Parameter::Gate(true)));
    s.send_message(RtMessage::new("Filter1", Parameter::Depth(1.5)));
    println!("Synthesizer running. Press Ctrl+C to stop.");
    s.run()?;

    return Ok(());
}