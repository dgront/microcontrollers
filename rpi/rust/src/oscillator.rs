use std::f64::consts::TAU;

use crate::settings::{CHANNELS, MASTER_VOLUME, SAMPLING_RATE};

pub struct Oscillator {
    phase: f64,
    frequency: f64,
}

impl Oscillator {
    pub fn new(frequency: f64) -> Self {
        Self {
            phase: 0.0,
            frequency,
        }
    }

    pub fn set_frequency(&mut self, frequency: f64) {
        self.frequency = frequency;
    }

    pub fn sample(&mut self) -> f32 {

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