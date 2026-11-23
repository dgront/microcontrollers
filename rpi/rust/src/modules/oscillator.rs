use std::collections::HashMap;
use log::info;
use crate::Module;
use crate::modules::{calculate_frequency, Input, Parameter};
use crate::sound::SoundWave;

pub struct Oscillator {
    /// The frequency ratio relative to the base frequency.
    ///
    /// To detune this oscillator, use number slightly above or below 1.0, e.g., 0.99 or 1.01.
    /// To change octave, use 0.5, or 2.0 for one octave down or up, respectively.
    frequency_ratio: f64,
    frequency: f64,
    gate: bool,
    phase: f64,
    waveform: SoundWave,
    name: String,
}

impl Module for Oscillator {
    fn set_parameter(&mut self, value: Parameter) {
        match value {
            Parameter::Frequency(freq) => self.set_frequency(freq),
            Parameter::FrequencyRatio(ratio) => self.set_frequency_ratio(ratio),
            Parameter::Gate(state) => self.set_gate(state),
            Parameter::Waveform(waveform) => self.set_waveform(waveform),
            _ => {}
        }
    }

    fn name(&self) -> &str { &self.name }
    
    fn sample(&mut self, inputs: &HashMap<Input, &[f32]>, output: &mut [f32]) {
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

    pub fn set_frequency(&mut self, new_freq: f64) {
        self.frequency = calculate_frequency(new_freq, 1.0, self.frequency_ratio);
        info!("Oscillator {} frequency set to {} Hz", self.name, self.frequency);
    }

    pub fn set_waveform(&mut self, waveform: SoundWave) {
        self.waveform = waveform;
        info!("Oscillator {} waveform set to {:?}", self.name, waveform);
    }

    pub fn set_gate(&mut self, state: bool) {
        self.gate = state;
        if self.gate {
            self.phase = 0.0; // Reset phase on gate open
        }
        info!("Oscillator {} gate set to {}", self.name, state);
    }

    pub fn set_frequency_ratio(&mut self, new_ratio: f64) {

        self.frequency_ratio = new_ratio;
        self.frequency = calculate_frequency(self.frequency, 1.0, self.frequency_ratio);
        info!("Oscillator {} frequency_ratio set to {} and frequency to {} Hz",
            self.name, self.frequency_ratio, self.frequency);
    }

    pub(super) fn from_attrs(frequency_ratio: f64, frequency: f64, gate: bool, waveform: SoundWave, name: String) -> Self {

        Self { frequency_ratio, frequency, gate, phase: 0.0, waveform, name }
    }
}