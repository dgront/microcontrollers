
use std::collections::HashMap;
use log::info;
use crate::Module;
use crate::modules::{Input, Parameter};

pub struct VCA {
    /// Gain multiplier: 0.0 = silence, 1.0 = unity gain.
    ///
    /// Values above 1.0 amplify the input signal.
    gain: f64,
    name: String,
}

impl Module for VCA {
    fn set_parameter(&mut self, value: Parameter) {
        match value {
            Parameter::Gain(gain) => self.set_gain(gain),
            _ => {}
        }
    }

    fn name(&self) -> &str { &self.name }

    fn sample(&mut self, inputs: &HashMap<Input, &[f32]>, output: &mut [f32]) {

        let audio = inputs.get(&Input::AudioIn).expect("VCA requires AudioIn buffer");

        assert_eq!(audio.len(), output.len());

        match inputs.get(&Input::Gain) {
            Some(gain_input) if !gain_input.is_empty() => {
                self.apply_modulated(audio, gain_input, output);
            }
            _ => {
                self.apply(audio, output);
            }
        }
    }
}

impl VCA {
    pub fn new(name: impl Into<String>) -> Self {
        Self { gain: 1.0, name: name.into() }
    }

    pub fn set_gain(&mut self, gain: f64) {
        self.gain = gain.max(0.0);
        info!("VCA {} gain set to {}", self.name, self.gain);
    }

    pub fn apply(&self, audio: &[f32], output: &mut [f32]) {
        for (sample, &input) in output.iter_mut().zip(audio) {
            *sample = input * self.gain as f32;
        }
    }

    pub fn apply_modulated(&self, audio: &[f32], modulation: &[f32], output: &mut [f32]) {

        assert_eq!(modulation.len(), output.len());

        for ((sample, &input), &gain) in output.iter_mut().zip(audio).zip(modulation) {
            *sample = input * self.gain as f32 * gain;
        }
    }

    pub(super) fn from_attrs(gain: f64, name: String) -> Self {
        Self { gain: gain.max(0.0), name }
    }
}
