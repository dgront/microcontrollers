
use std::sync::mpsc::SyncSender;
use std::thread;
use std::time::{Duration, Instant};

use crate::modules::{Parameter, RtMessage};

#[derive(Clone, Copy)]
pub struct Step {
    pub frequency: f64,
    pub gate: f64, // 0.0 = rest, 1.0 = full step
}

pub struct Sequencer {
    tx: SyncSender<RtMessage>,
    frequency_ports: Vec<String>,
    gate_ports: Vec<String>,
    steps: Vec<Step>,
    bpm: f64,
    beats_per_bar: u32,
}

impl Sequencer {
    pub fn new(tx: SyncSender<RtMessage>) -> Self {
        Self {
            tx,
            frequency_ports: Vec::new(),
            gate_ports: Vec::new(),
            steps: vec![Step { frequency: 0.0, gate: 0.0}; 8],
            bpm: 120.0,
            beats_per_bar: 4,
        }
    }

    pub fn connect_frequency(&mut self, name: impl Into<String>) {
        self.frequency_ports.push(name.into());
    }

    pub fn connect_gate(&mut self, name: impl Into<String>) {
        self.gate_ports.push(name.into());
    }

    pub fn set_bpm(&mut self, bpm: f64) {
        assert!(bpm > 0.0);
        self.bpm = bpm;
    }

    pub fn set_steps_per_bar(&mut self, count: usize) {

        assert!(count > 0);
        self.steps.resize(count, Step { frequency: 0.0, gate: 0.0});
    }

    pub fn set_step(&mut self, index: usize, frequency: f64, gate: f64) {
        self.steps[index] = Step {
            frequency,
            gate: gate.clamp(0.0, 1.0),
        };
    }

    pub fn set_steps(&mut self, steps: Vec<(f64, f64)>) {
        assert!(!steps.is_empty(), "Sequencer requires at least one step");

        self.steps = steps
            .into_iter()
            .map(|(frequency, gate)| Step { frequency, gate: gate.clamp(0.0, 1.0)}).collect();
    }

    fn send_frequency(&self, frequency: f64) {
        for module in &self.frequency_ports {
            let _ = self.tx.send(
                RtMessage::new(
                    module.clone(),
                    Parameter::Frequency(frequency),
                )
            );
        }
    }

    fn send_gate(&self, state: bool) {
        for module in &self.gate_ports {
            let _ = self.tx.send(
                RtMessage::new(
                    module.clone(),
                    Parameter::Gate(state),
                )
            );
        }
    }

    pub fn run(&self) {
        let step_duration = Duration::from_secs_f64(
            60.0 / self.bpm * self.beats_per_bar as f64 / self.steps.len() as f64
        );

        let mut gate_open = false;
        let mut next_step = Instant::now();

        loop {
            for step in &self.steps {
                let gate_duration = step_duration.mul_f64(step.gate);

                if step.gate > 0.0 {
                    // Update pitch even when the gate is already open.
                    self.send_frequency(step.frequency);

                    if !gate_open {
                        self.send_gate(true);
                        gate_open = true;
                    }
                } else if gate_open {
                    // Rest: close the gate.
                    self.send_gate(false);
                    gate_open = false;
                }

                // Wait until the gate should close.
                next_step += gate_duration;
                let now = Instant::now();

                if next_step > now {
                    thread::sleep(next_step - now);
                }

                if step.gate < 1.0 && gate_open {
                    self.send_gate(false);
                    gate_open = false;
                }

                // Wait until the next step.
                next_step += step_duration - gate_duration;
                let now = Instant::now();

                if next_step > now {
                    thread::sleep(next_step - now);
                }
            }
        }
    }
}
