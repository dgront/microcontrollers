use std::collections::HashMap;
use crate::modules::{Input, Module, Parameter};
use crate::settings::SAMPLING_RATE;


macro_rules! filter_sample {
    () => {
        fn sample(&mut self, inputs: HashMap<Input, &[f32]>, output: &mut [f32]) {
            let audio = inputs.get(&Input::AudioIn).expect("Filter requires AudioIn");

            output.copy_from_slice(audio);
            self.apply(output);
        }
    };
}

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

    filter_sample!();
}


#[derive(Debug, Clone)]
pub struct BiquadFilter {

    cutoff: f64,
    resonance: f64,

    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
    name: String
}

impl BiquadFilter {
    pub fn new(name: impl Into<String>, cutoff: f64, resonance: f64) -> Self {
        let mut filter = Self {
            name: name.into(), cutoff, resonance,
            b0: 0.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, z1: 0.0, z2: 0.0
        };

        filter.update_coefficients();
        return filter;
    }

    pub fn set_cutoff(&mut self, cutoff: f64) {
        self.cutoff = cutoff;
        self.update_coefficients();
    }

    pub fn set_resonance(&mut self, resonance: f64) {
        self.resonance = resonance;
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        let w = 2.0 * std::f64::consts::PI * self.cutoff / SAMPLING_RATE as f64;

        let (sin, cos) = w.sin_cos();
        let alpha = sin / (2.0 * self.resonance);
        let a0 = 1.0 + alpha;

        self.b0 = ((1.0 - cos) / (2.0 * a0)) as f32;
        self.b1 = ((1.0 - cos) / a0) as f32;
        self.b2 = self.b0;
        self.a1 = (-2.0 * cos / a0) as f32;
        self.a2 = ((1.0 - alpha) / a0) as f32;
    }

    pub fn apply(&mut self, samples: &mut [f32]) {

        for x in samples {
            let y = self.b0 * *x + self.z1;
            self.z1 = self.b1 * *x - self.a1 * y + self.z2;
            self.z2 = self.b2 * *x - self.a2 * y;
            *x = y;
        }
    }
}

impl Module for BiquadFilter {
    fn set_parameter(&mut self, value: Parameter) {
        match value {
            Parameter::Frequency(frequency) => {
                self.set_cutoff(frequency);
            }
            Parameter::Resonance(resonance) => {
                self.set_resonance(resonance);
            }
            _ => {}
        }
    }

    fn name(&self) -> &str { return &self.name; }

    filter_sample!();
}