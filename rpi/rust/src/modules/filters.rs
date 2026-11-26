use std::collections::HashMap;
use crate::modules::{Input, Module, Parameter};
use crate::settings::SAMPLING_RATE;


macro_rules! filter_sample {
    () => {
        fn sample(&mut self, inputs: &HashMap<Input, &[f32]>, output: &mut [f32]) {

            let audio = inputs.get(&Input::AudioIn).expect("Filter requires AudioIn");
            output.copy_from_slice(audio);
            self.apply(output);

            match inputs.get(&Input::Frequency) {
                Some(freq_input) if !freq_input.is_empty() => {
                    self.apply_modulated(freq_input, output);
                }
                _ => {
                    self.apply(output);
                }
            }

        }
    };
}


///
/// \[
// f_c(t)=f_{c,0}\cdot 2^{dE(t)}
// \]
#[inline]
pub fn modulated_frequency(base_frequency: f64, depth: f64, modulation: f64) -> f64 {
    base_frequency * (depth * modulation).exp2()
}

#[derive(Debug, Clone)]
pub struct LowPassFilter {
    cutoff: f64,
    depth: f64,
    alpha: f32,
    previous: f32,
    name: String
}

impl LowPassFilter {
    pub fn new(name: impl Into<String>, cutoff: f64) -> Self {
        let alpha = (1.0 - (-2.0 * std::f64::consts::PI * cutoff / SAMPLING_RATE as f64).exp()) as f32;

        Self { cutoff, depth: 1.0, alpha, previous: 0.0, name: name.into() }
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

    pub fn apply_modulated(&mut self, modulation: &[f32], samples: &mut [f32]) {

        for (i, x) in samples.iter_mut().enumerate() {
            let cutoff = modulated_frequency(self.cutoff, self.depth, modulation[i] as f64);
            let alpha = Self::evaluate_alpha(cutoff);
            self.previous += alpha * (*x - self.previous);
            *x = self.previous;
        }
    }

    fn evaluate_alpha(new_cutoff: f64) -> f32 {
        (1.0 - (-2.0 * std::f64::consts::PI * new_cutoff / SAMPLING_RATE as f64).exp()) as f32
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

    /// Cut-off frequency for this filter
    cutoff: f64,
    /// Resonance value
    resonance: f64,
    /// When the ``cutoff`` value is controlled by [`Input::Frequency`] buffer, ``depth`` scales the amount of the modulation
    depth: f64,

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
            name: name.into(), cutoff, resonance, depth: 1.0,
            b0: 0.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, z1: 0.0, z2: 0.0
        };

        filter.update_coefficients(cutoff, resonance);
        return filter;
    }

    /// Depth controls how much LFO modulation affects the filter cutoff
    ///
    pub fn set_depth(&mut self, depth: f64) {
        self.depth = depth;
    }

    pub fn set_cutoff(&mut self, cutoff: f64) {
        self.cutoff = cutoff;
        self.update_coefficients(self.cutoff, self.resonance);
    }

    pub fn set_resonance(&mut self, resonance: f64) {
        self.resonance = resonance;
        self.update_coefficients(self.cutoff, self.resonance);
    }

    /// Updates the coefficients used by this filter based on the provided cutoff frequency and resonance.
    ///
    /// The ``cutoff`` and resonance ``fields`` of this object are not affected
    fn update_coefficients(&mut self, cutoff: f64, resonance: f64) {
        let w = 2.0 * std::f64::consts::PI * cutoff / SAMPLING_RATE as f64;

        let (sin, cos) = w.sin_cos();
        let alpha = sin / (2.0 * resonance);
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

    pub fn apply_modulated(&mut self, modulation: &[f32], samples: &mut [f32]) {

        for (i, x) in samples.iter_mut().enumerate() {
            let cutoff = modulated_frequency(self.cutoff, self.depth, modulation[i] as f64);
            self.update_coefficients(cutoff, self.resonance);
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