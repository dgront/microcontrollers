use std::collections::HashMap;
use log::info;
use crate::modules::{Input, Module, Parameter};
use crate::sound::SoundWave;

pub struct LFO {
    phase: f64,
    frequency: f64,
    depth: f32,
    gate: bool,
    waveform: SoundWave,
    name: String,
}

impl LFO {
    pub fn new(name: String) -> Self {
        Self { phase: 0.0,  frequency: 1.0, name, depth: 0.0 , gate: false, waveform: SoundWave::Sine}
    }
}

impl Module for LFO {

    fn set_parameter(&mut self, value: Parameter) {
        match value {
            Parameter::Frequency(freq) => {
                self.frequency = freq;
                info!("LFO {} frequency set to {} Hz", self.name, self.frequency);
            },
            Parameter::Gate(state) => {
                self.gate = state;
                if self.gate {
                    self.phase = 0.0; // Reset phase on gate open
                }
                info!("LFO {} gate set to {}", self.name, state);
            },
            Parameter::Waveform(waveform) => {
                info!("LFO {} waveform set to {:?}", self.name, waveform);
            },
            Parameter::Depth(depth) => {
                self.depth = depth;
                info!("LFO {} depth set to {}", self.name, self.depth);
            },
            _ => {}
        }
    }

    fn name(&self) -> &str { &self.name }

    /// LFO doesn't accept any input at that moment
    fn sample(&mut self, _inputs: HashMap<Input, &[f32]>, output: &mut [f32]) {
        // ---------- Generate the LFO waveform
        self.waveform.sample(&mut self.phase, self.frequency, output);
        // ---------- Apply depth to the output
        for sample in output.iter_mut() {
            *sample *= self.depth;
        }
    }
}