use std::collections::HashMap;
use crate::modules::{Input, Module, Parameter};
use crate::settings::SAMPLING_RATE;

#[derive(Debug, Clone)]
pub struct LowPassFilter {
    alpha: f32,
    previous: f32,
    name: String
}

impl LowPassFilter {
    pub fn new(name: impl Into<String>, cutoff: f64) -> Self {
        let alpha = (1.0 - (-2.0 * std::f64::consts::PI * cutoff / SAMPLING_RATE as f64).exp()) as f32;

        Self { alpha, previous: 0.0, name: name.into() }
    }

    pub fn set_cutoff(&mut self, new_cutoff: f64) {
        self.alpha = (1.0 - (-2.0 * std::f64::consts::PI * new_cutoff / SAMPLING_RATE as f64).exp()) as f32;
    }

    pub fn apply(&mut self, samples: &mut [f32]) {
        for x in samples {
            self.previous += self.alpha * (*x - self.previous);
            *x = self.previous;
        }
    }
}

impl Module for LowPassFilter {
    fn set_parameter(&mut self, value: Parameter) {
        match value {
            Parameter::Frequency(frequency) => {
                self.set_cutoff(frequency);
            }
            _ => {}
        }
    }

    fn name(&self) -> &str { return &self.name; }

    fn sample(&mut self, inputs: HashMap<Input, &[f32]>, output: &mut [f32]) {

        // ---------- Get the audio input.
        let audio = inputs.get(&Input::AudioIn).expect("LowPassFilter requires AudioIn");
        // ---------- Copy the input signal into the output buffer.
        output.copy_from_slice(audio);

        // ---------- Apply the envelope in place.
        self.apply(output);
    }
}