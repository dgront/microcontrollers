use std::f64::consts::TAU;
use log::info;
use crate::Module;
use crate::module::Parameter;
use crate::settings::{CHANNELS, MASTER_VOLUME, SAMPLING_RATE, BUFFER_FRAMES};

pub struct Oscillator {
    phase: f64,
    frequency_ratio: f64,
    frequency: f64,
    name: String,
    gate: bool,
}

impl Module for Oscillator {
    fn name(&self) -> &str { &self.name }

    fn set_parameter(&mut self, value: Parameter) {
        match value {
            Parameter::Frequency(freq) => {
                self.frequency = freq as f64;
                info!("Oscillator {} frequency set to {} Hz", self.name, self.frequency);
            },
            Parameter::Gate(state) => {
                self.gate = state;
                if self.gate {
                    self.phase = 0.0; // Reset phase on gate open
                }
                info!("Oscillator {} gate set to {}", self.name, state);
            },
            _ => {}
        }
    }
}

impl Oscillator {
    pub fn new(name: String) -> Self {
        Self { phase: 0.0, frequency_ratio: 1.0, frequency: 440.0, name, gate: false }
    }

    #[inline(always)]
    pub fn sample(&mut self) -> f32 {

        if !self.gate { return 0.0; }

        let value = (TAU * self.phase).sin() as f32;

        self.phase = (self.phase + self.frequency / SAMPLING_RATE).fract();

        value * MASTER_VOLUME
    }

    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for frame in buffer.chunks_exact_mut(CHANNELS) {
            let sample = self.sample();
            frame.fill(sample);
        }
    }
}