use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PortId {
    pub module: String,
}

impl PortId {
    pub fn new(module: &str) -> Self {
        Self { module: module.to_owned() }
    }
}

impl fmt::Display for PortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.module)
    }
}

impl FromStr for PortId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        Ok(Self { module: s.to_owned() })
    }
}

pub enum Parameter {
    Frequency(f32),
    Volume(f32),
    Waveform(SoundWave),
    Gate(bool),
}

pub enum SoundWave {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

pub struct RtMessage {
    pub port: PortId,
    pub value: Parameter,
}

pub trait Module {
    fn set_parameter(&mut self, value: Parameter);

    fn name(&self) -> &str;

    fn id(&self) -> PortId {
        PortId::new(self.name())
    }
}