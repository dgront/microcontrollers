
use std::collections::HashMap;
use std::f64::consts::TAU;

use log::info;

use crate::Module;
use crate::modules::{Input, Parameter};
use crate::settings::SAMPLING_RATE;

const MAX_DELAY_MS: f64 = 30.0;
const BASE_DELAY_MS: f64 = 15.0;
const MODULATION_MS: f64 = 10.0;

pub struct Chorus {
    name: String,
    frequency: f64,
    depth: f64,
    mix: f32,
    phase: f64,

    delay: Vec<f32>,
    write_pos: usize,
}

impl Module for Chorus {
    fn name(&self) -> &str { &self.name }

    fn set_parameter(&mut self, value: Parameter) {
        match value {
            Parameter::Frequency(freq) => self.set_frequency(freq),
            Parameter::Depth(depth) => self.set_depth(depth),
            Parameter::Mix(mix) => self.set_mix(mix),
            _ => {}
        }
    }

    fn sample(&mut self, inputs: &HashMap<Input, &[f32]>, output: &mut [f32]) {

        let audio = inputs.get(&Input::AudioIn).expect("Chorus requires AudioIn");

        assert_eq!(audio.len(), output.len());

        let modulation = inputs.get(&Input::Frequency).copied();

        if let Some(modulation) = modulation {
            assert_eq!(modulation.len(), output.len());
        }

        let sample_rate = SAMPLING_RATE as f64;
        let base_delay = BASE_DELAY_MS * sample_rate / 1000.0;
        let modulation_range = MODULATION_MS * sample_rate / 1000.0;

        let phase_step = self.frequency / sample_rate;
        let delay_len = self.delay.len();

        for i in 0..output.len() {
            let input = audio[i];

            // Use external LFO if connected, otherwise internal sine LFO.
            let lfo = match modulation {
                Some(buffer) => buffer[i].clamp(-1.0, 1.0) as f64,
                None => (TAU * self.phase).sin(),
            };

            // Variable delay in samples.
            let delay_samples =
                base_delay + self.depth * modulation_range * lfo;

            // Fractional read position in circular delay buffer.
            let read_pos = (self.write_pos as f64 - delay_samples).rem_euclid(delay_len as f64);

            let index0 = read_pos as usize;
            let index1 = (index0 + 1) % delay_len;
            let fraction = (read_pos - index0 as f64) as f32;

            // Linear interpolation between adjacent samples.
            let delayed = self.delay[index0] * (1.0 - fraction) + self.delay[index1] * fraction;

            // Store input in delay line.
            self.delay[self.write_pos] = input;
            self.write_pos = (self.write_pos + 1) % delay_len;

            // Mix original and delayed signals.
            output[i] = input * (1.0 - self.mix) + delayed * self.mix;

            // Advance internal LFO phase.
            if modulation.is_none() {
                self.phase += phase_step;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
            }
        }
    }
}

impl Chorus {
    pub fn new(name: impl Into<String>) -> Self {
        let delay_len =
            (MAX_DELAY_MS * SAMPLING_RATE as f64 / 1000.0)
                .ceil() as usize + 2;

        Self {
            name: name.into(),
            frequency: 0.5,
            depth: 0.5,
            mix: 0.5,
            phase: 0.0,
            delay: vec![0.0; delay_len],
            write_pos: 0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f64) {
        self.frequency = frequency.clamp(0.0, 20.0);
        info!("Chorus {} frequency set to {} Hz",self.name, self.frequency);
    }

    pub fn set_depth(&mut self, depth: f64) {
        self.depth = depth.clamp(0.0, 1.0);
        info!("Chorus {} depth set to {}",self.name, self.depth);
    }

    pub fn set_mix(&mut self, mix: f64) {
        self.mix = mix.clamp(0.0, 1.0) as f32;
        info!("Chorus {} mix set to {}",self.name, self.mix);
    }
}
