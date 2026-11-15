use std::f64::consts::TAU;
use log::info;
use crate::Module;
use crate::modules::Parameter;
use crate::settings::{CHANNELS, MASTER_VOLUME, SAMPLING_RATE, BUFFER_FRAMES};

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
            _ => {}
        }
    }

    fn name(&self) -> &str { &self.name }
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