use std::collections::HashMap;
use std::fmt;
use log::debug;
use crate::modules::{Input, Module, Parameter};
use crate::settings::{DT};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState {
    Attack { slope: f64, duration: f64 },
    Decay { slope: f64, duration: f64 },
    Sustain { slope: f64, duration: f64 },
    Release { slope: f64, duration: f64 },
    Iddle,
    Retrigger { slope: f64, duration: f64 },
}

impl fmt::Display for EnvelopeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Attack { slope, duration } => {
                write!(f, "Attack(slope={slope}, duration={duration})")
            }
            Self::Decay { slope, duration } => {
                write!(f, "Decay(slope={slope}, duration={duration})")
            }
            Self::Sustain { slope, duration } => {
                write!(f, "Sustain(slope={slope}, duration={duration})")
            }
            Self::Release { slope, duration } => {
                write!(f, "Release(slope={slope}, duration={duration})")
            }
            Self::Iddle => write!(f, "Idle"),
            Self::Retrigger { slope, duration } => {
                write!(f, "Retrigger(slope={slope}, duration={duration})")
            }
        }
    }
}

impl EnvelopeState {
    fn slope(&self) -> f64 {
        match self {
            EnvelopeState::Attack { slope,.. } => { *slope }
            EnvelopeState::Decay { slope,.. } => { *slope }
            EnvelopeState::Sustain { slope,.. } => { *slope }
            EnvelopeState::Release { slope,.. } => { *slope }
            EnvelopeState::Iddle  => { 0.0 }
            EnvelopeState::Retrigger { slope,.. } => { *slope }
        }
    }
    fn duration(&self) -> f64 {
        match self {
            EnvelopeState::Attack { duration, .. } => { *duration }
            EnvelopeState::Decay { duration,.. } => { *duration }
            EnvelopeState::Sustain { duration,.. } => { *duration }
            EnvelopeState::Release { duration,.. } => { *duration }
            EnvelopeState::Iddle  => { 100000.0 }
            EnvelopeState::Retrigger { duration,.. } => { *duration }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ADSREnvelope {
    stages: [EnvelopeState; 6],
    current_state_idx: usize,
    current_stage_time: f64,
    current_stage_level: f64,
    gate: bool,
    name: String,
}

impl ADSREnvelope {
    pub fn new(name: impl Into<String>, attack_time: f64, decay_time: f64, decay_level: f64,
               sustain_time: f64, sustain_level: f64, release_time: f64) -> Self {
        
        let dl = decay_level.clamp(0.0, 1.0);
        let sl = sustain_level.clamp(0.0, 1.0);

        let slope = |from: f64, to: f64, duration: f64| {
            if duration > 0.0 {
                (to - from) / (duration/DT)
            } else { 0.0 }
        };

        Self {
            stages: [
                EnvelopeState::Attack {
                    slope: slope(0.0, 1.0, attack_time),
                    duration: attack_time,
                },
                EnvelopeState::Decay {
                    slope: slope(1.0, dl, decay_time),
                    duration: decay_time,
                },
                EnvelopeState::Sustain {
                    slope: slope(dl, sl, sustain_time),
                    duration: sustain_time,
                },
                EnvelopeState::Release {
                    slope: slope(sl, 0.0, release_time),
                    duration: release_time,
                },
                EnvelopeState::Iddle,
                EnvelopeState::Retrigger {
                    slope: slope(sl, 0.0, 0.003),
                    duration: 0.003,
                },
            ],
            current_state_idx: 0, // Start!
            current_stage_time: 0.0,
            current_stage_level: 0.0,
            gate: false,
            name: name.into(),
        }
    }

    pub fn start(&mut self) {

        if matches!(self.current_state(), EnvelopeState::Iddle) {
            self.current_state_idx = 0;
            self.current_stage_time = 0.0;
            self.current_stage_level = 0.0;
            self.gate = true;
            return;
        }

        let retrigger_idx = self.stages.len() - 1;
        let duration = self.stages[retrigger_idx].duration();

        let slope = if duration > 0.0 {
            -self.current_stage_level * DT / duration
        } else {
            0.0
        };

        self.stages[retrigger_idx] = EnvelopeState::Retrigger { slope, duration};

        self.current_state_idx = retrigger_idx;
        self.current_stage_time = 0.0;
        self.gate = true;
    }

    pub fn current_state(&self) -> &EnvelopeState {
        return &self.stages[self.current_state_idx];
    }

    pub fn release(&mut self) {

        if self.current_state_idx >= 3 { return; }
        // Release if not released yet
        let duration = self.stages[3].duration();

        let slope = if duration > 0.0 {
            -self.current_stage_level * DT / duration
        } else {
            0.0
        };

        self.stages[3] = EnvelopeState::Release { slope, duration};
        self.current_state_idx = 3;
        self.current_stage_time = 0.0;
        self.gate = false;
    }

    pub fn apply(&mut self, out: &mut [f32]) {

        let mut step: usize = 0;
        let mut current_slope = self.current_state().slope();
        let mut current_duration = self.current_state().duration();
        while step < out.len() {
            self.current_stage_level += current_slope;
            out[step] *= self.current_stage_level as f32;
            self.current_stage_time += DT;
            if self.current_stage_time >= current_duration {
                if matches!(self.current_state(), EnvelopeState::Retrigger { .. }) {
                    self.current_state_idx = 0;
                    self.current_stage_level = 0.0;
                } else if !matches!(self.current_state(), EnvelopeState::Iddle) {
                    self.current_state_idx += 1;

                    if matches!(self.current_state(), EnvelopeState::Iddle) {
                        self.current_stage_level = 0.0;
                    }
                }

                current_duration = self.current_state().duration();
                current_slope = self.current_state().slope();
                self.current_stage_time = 0.0;

                debug!("ADSREnvelope entered {} stage", self.current_state());
            }
            step += 1;
        }
    }
}

impl Module for ADSREnvelope {
    fn set_parameter(&mut self, value: Parameter) {
        match value {
            Parameter::Gate(state) => {
                if state { self.start() }
                else { self.release() }
            }
            _ => {}
        }
    }

    fn name(&self) -> &str { return &self.name; }

    fn sample(&mut self, inputs: HashMap<Input, &[f32]>, output: &mut [f32]) {

        // ---------- Get the audio input.
        let audio = inputs.get(&Input::AudioIn).expect("Envelope requires AudioIn");
        // ---------- Copy the input signal into the output buffer.
        output.copy_from_slice(audio);

        // ---------- Apply the envelope in place.
        self.apply(output);
    }
}

impl fmt::Display for ADSREnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut t_start = 0.0;
        for stage in &self.stages {
            let (name, slope, duration) = match stage {
                EnvelopeState::Attack { slope, duration } =>
                    ("Attack", *slope, *duration),
                EnvelopeState::Decay { slope, duration } =>
                    ("Decay", *slope, *duration),
                EnvelopeState::Sustain { slope, duration } =>
                    ("Sustain", *slope, *duration),
                EnvelopeState::Release { slope, duration } =>
                    ("Release", *slope, *duration),
                EnvelopeState::Iddle => continue,
                EnvelopeState::Retrigger{ slope, duration } =>
                    ("Release", *slope, *duration),
            };

            let t_stop = t_start + duration;
            writeln!(f, "{name:7}: from {t_start:.3} to {t_stop:.3} [s], slope {:.3}/[ms]", (slope*1000.0))?;
            t_start = t_stop;
        }

        let current = match self.current_state_idx {
            0 => "Attack",
            1 => "Decay",
            2 => "Sustain",
            3 => "Release",
            _ => "Idle",
        };

        write!(f, "Currently at: {current:7}")
    }
}
