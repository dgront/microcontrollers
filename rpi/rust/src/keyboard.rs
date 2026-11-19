use std::collections::HashSet;
use std::sync::mpsc::SyncSender;
use log::info;
use crate::modules::{RtMessage};
use crate::modules::Parameter::{Frequency, Gate};
use crate::notes::Note;

pub struct Keyboard {
    tx: SyncSender<RtMessage>,
    frequency_ports: HashSet<String>,
    gate_ports: HashSet<String>,
    octave: u16
}

impl Keyboard {
    pub fn new(tx: SyncSender<RtMessage>) -> Self {
        Self { tx, frequency_ports: HashSet::new(), gate_ports: HashSet::new(), octave: 4 }
    }

    pub fn run(&self) -> Result<(), String> {
        use std::io::{self, BufRead};

        for line in io::stdin().lock().lines().flatten() {
            info!("Keyboard input: {}", line.trim());
            let name = "A";
            if line.trim() == "a" {
                let note = name.parse::<Note>().map_err(|_| format!("Invalid note: {name}"))?;
                self.send_note_on(note.octave_frequency(self.octave));
            }
            if line.trim() == "" {
                // Close gates.
                for module in &self.gate_ports {
                    let _ = self.tx.try_send(RtMessage::new(module.clone(), Gate(false)));
                }
            }
        }

        Ok(())
    }

    pub fn connect_frequency(&mut self, module: impl Into<String>) {
        self.frequency_ports.insert(module.into());
    }

    pub fn connect_gate(&mut self, module: impl Into<String>) {
        self.gate_ports.insert(module.into());
    }

    pub fn disconnect_frequency(&mut self, module: &str) {
        self.frequency_ports.remove(module);
    }

    pub fn disconnect_gate(&mut self, module: &str) {
        self.gate_ports.remove(module);
    }

    fn send_note_on(&self, frequency: f64) {
        for module in &self.frequency_ports {
            let _ = self.tx.try_send(RtMessage::new(module.clone(), Frequency(frequency)));
        }
        for module in &self.gate_ports {
            let _ = self.tx.try_send(RtMessage::new(module.clone(), Gate(true)));
        }
    }
}