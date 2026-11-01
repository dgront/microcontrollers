use std::fmt;

use crate::settings::{DT};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState {
    Attack { slope: f64, duration: f64 },
    Decay { slope: f64, duration: f64 },
    Sustain { slope: f64, duration: f64 },
    Release { slope: f64, duration: f64 },
    Iddle
}

impl EnvelopeState {
    fn slope(&self) -> f64 {
        match self {
            EnvelopeState::Attack { slope,.. } => { *slope }
            EnvelopeState::Decay { slope,.. } => { *slope }
            EnvelopeState::Sustain { slope,.. } => { *slope }
            EnvelopeState::Release { slope,.. } => { *slope }
            EnvelopeState::Iddle  => { 0.0 }
        }
    }
    fn duration(&self) -> f64 {
        match self {
            EnvelopeState::Attack { duration, .. } => { *duration }
            EnvelopeState::Decay { duration,.. } => { *duration }
            EnvelopeState::Sustain { duration,.. } => { *duration }
            EnvelopeState::Release { duration,.. } => { *duration }
            EnvelopeState::Iddle  => { 100000.0 }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ADSREnvelope {
    stages: [EnvelopeState; 5],
    current_state_idx: usize,
    current_stage_time: f64,
    current_stage_level: f64,
}

impl ADSREnvelope {
    pub fn new(attack_time: f64, decay_time: f64, decay_level: f64, sustain_time: f64, sustain_level: f64, release_time: f64) -> Self {
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
                EnvelopeState::Iddle
            ],
            current_state_idx: 0, // Start!
            current_stage_time: 0.0,
            current_stage_level: 0.0,
        }
    }

    pub fn current_state(&self) -> &EnvelopeState {
        return &self.stages[self.current_state_idx];
    }

    pub fn release(&mut self) {
        self.current_state_idx = 3; // Release
        self.current_stage_time = 0.0;
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
                self.current_state_idx += 1;
                current_duration = self.current_state().duration();
                current_slope = self.current_state().slope();
                self.current_stage_time = 0.0;
            }
            step += 1;
        }
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
