use std::collections::HashMap;
use crate::sound::SoundWave;

#[derive(Debug, Clone, Copy)]
pub enum Parameter {
    Frequency(f64),
    FrequencyRatio(f64),
    Volume(f64),
    Waveform(SoundWave),
    Gate(bool),
    Depth(f64),
    Gain(f64),
    Resonance(f64),
    Mix(f64),
}


/// Represents a message sent to a module in the synthesizer.
///
/// The `RtMessage` struct contains the name of the receiving module and the parameter value
/// to be set on that module. This allows for real-time control of scalar module parameters
/// (such as gate state or volume value) during audio processing. Once constructed, such a message
/// may be sent by [`Synthesizer::send_message()](crate::synthesizer::Synthesizer::send_message) method.
/// Vector parameters (like audio buffers) are not sent via messages, but rather through the input/output connections
/// of the modules.
///
/// # Example
/// ```
/// let msg = RtMessage::new("Oscillator1", Parameter::Frequency(440.0));
/// ```
pub struct RtMessage {
    /// The name of the module that should receive this message.
    pub receiver_module: String,
    /// The parameter value to be set on the receiving module.
    ///
    /// This provides both the value and the type of parameter being set, e.g. ``Parameter::Frequency(440.0)`` or ``Parameter::Gate(true)``.
    pub value: Parameter,
}

impl RtMessage {
    /// Creates a new `RtMessage` with the specified receiver module and parameter value.
    pub fn new(receiver_module: impl Into<String>, value: Parameter) -> Self {
        Self {
            receiver_module: receiver_module.into(),
            value,
        }
    }
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

    fn sample(&mut self, inputs: &HashMap<Input, &[f32]>, output: &mut [f32]);
}