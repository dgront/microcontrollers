use std::collections::HashMap;
use crate::sound::SoundWave;

#[derive(Debug, Clone, Copy)]
pub enum Parameter {
    Frequency(f64),
    FrequencyRatio(f64),
    Volume(f64),
    Waveform(SoundWave),
    Gate(bool),
    Depth(f32),
}


pub struct RtMessage {
    pub receiver_module: String,
    pub value: Parameter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Input {
    AudioIn,
    AudioOut,
    Frequency,
    Gain,
    Resonance
}

pub trait Module {
    fn set_parameter(&mut self, value: Parameter);

    fn name(&self) -> &str;

    fn sample(&mut self, inputs: HashMap<Input, &[f32]>, output: &mut [f32]);
}