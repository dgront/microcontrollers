use std::collections::HashMap;
use log::info;
use crate::Module;
use crate::modules::{Input, Oscillator, Parameter};
use crate::sound::SoundWave;

pub struct UnisonOscillator {
    name: String,
    oscillators: Vec<Oscillator>
}

impl Module for UnisonOscillator {
    fn set_parameter(&mut self, value: Parameter) {
        match value {
            Parameter::Frequency(freq) => {
                info!("Base frequency for {} set to {} Hz", self.name, freq);
                for o in &mut self.oscillators {
                    o.set_frequency(freq);
                }
            },
            Parameter::Gate(state) => {
                for o in &mut self.oscillators {
                    o.set_gate(state);
                }
            },
            Parameter::Waveform(waveform) => {
                for o in &mut self.oscillators {
                    o.set_waveform(waveform);
                }
            },
            _ => {}
        }
    }

    fn name(&self) -> &str { &self.name }

    fn sample(&mut self, inputs: &HashMap<Input, &[f32]>, output: &mut [f32]) {

        // ---------- Clear the output buffer before mixing; leave zeros when all the gates are closed
        output.fill(0.0);

        for o in &mut self.oscillators { o.sample(inputs, output); }

        for sample in output.iter_mut() {
            *sample /= self.oscillators.len() as f32; // Normalize the mixed output
        }
    }
}

impl UnisonOscillator {
    pub fn new(name: impl Into<String>, n_voices: usize, detune_fraction: f64) -> Self {

        let mut oscillators = Vec::with_capacity(n_voices);
        let name_s: String = name.into();
        for i in 0..n_voices {
            let spread_factor = if n_voices > 1 {
                (i as f64 / (n_voices - 1) as f64) * 2.0 - 1.0
            } else {
                0.0
            };
            // Detune offset for this voice, scaled by the spread factor
            let voice_detune_offset = spread_factor * detune_fraction;
            let name = format!("{}-{}", &name_s, i);
            oscillators.push(Oscillator::from_attrs(voice_detune_offset, 440.0, false, SoundWave::Sine, name));
        }

        Self { name: name_s, oscillators }
    }
}