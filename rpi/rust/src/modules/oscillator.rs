use std::collections::HashMap;
use log::info;
use crate::Module;
use crate::modules::{Input, Parameter};
use crate::sound::SoundWave;

pub struct Oscillator {
    phase: f64,
    /// The frequency ratio relative to the base frequency.
    ///
    /// To detune this oscillator, use number slightly above or below 1.0, e.g., 0.99 or 1.01.
    /// To change octave, use 0.5, or 2.0 for one octave down or up, respectively.
    frequency_ratio: f64,
    frequency: f64,
    name: String,
    gate: bool,
    waveform: SoundWave,
}

impl Module for Oscillator {
    fn set_parameter(&mut self, value: Parameter) {
        match value {
            Parameter::Frequency(freq) => {
                self.frequency = freq * self.frequency_ratio;
                info!("Oscillator {} frequency set to {} Hz", self.name, self.frequency);
            },
            Parameter::FrequencyRatio(ratio) => {
                self.frequency_ratio = ratio;
                self.frequency = self.frequency * self.frequency_ratio;
                info!("Oscillator {} frequency_ratio set to {} and frequency to {} Hz",
                    self.name, self.frequency_ratio, self.frequency);
            },
            Parameter::Gate(state) => {
                self.gate = state;
                if self.gate {
                    self.phase = 0.0; // Reset phase on gate open
                }
                info!("Oscillator {} gate set to {}", self.name, state);
            },
            Parameter::Waveform(waveform) => {
                self.waveform = waveform;
                info!("Oscillator {} waveform set to {:?}", self.name, waveform);
            },
            _ => {}
        }
    }

    fn name(&self) -> &str { &self.name }
    
    fn sample(&mut self, inputs: HashMap<Input, &[f32]>, output: &mut [f32]) {
        // ---------- Don't generate sound if the gate is closed
        if self.gate == false {
            output.fill(0.0);
            return;
        }

        match inputs.get(&Input::Frequency) {
            Some(freq_input) if !freq_input.is_empty() => {
                self.waveform.sample_modulated(&mut self.phase, self.frequency, freq_input, output);
            }
            _ => {
                self.waveform.sample(&mut self.phase, self.frequency, output);
            }
        }
    }
}

impl Oscillator {
    pub fn new(name: impl Into<String>) -> Self {
        Self { phase: 0.0, frequency_ratio: 1.0, frequency: 440.0, name: name.into(), gate: false, waveform: SoundWave::Sine}
    }
}