use log::info;
use crate::modules::{Module, Parameter, SoundWave};
use crate::modules::oscillator::Oscillator;

pub struct LFO {
    phase: f64,
    frequency: f64,
    waveform: SoundWave,
    gate: bool,
    name: String,
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
            _ => {}
        }
    }

    fn name(&self) -> &str { &self.name }
}