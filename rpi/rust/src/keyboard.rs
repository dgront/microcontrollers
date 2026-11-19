use std::collections::HashSet;
use std::sync::mpsc::SyncSender;
use log::info;
use crate::modules::{RtMessage};
use crate::modules::Parameter::{Frequency, Gate};

pub struct Keyboard {
    tx: SyncSender<RtMessage>,
    frequency_ports: HashSet<String>,
    gate_ports: HashSet<String>,
}

impl Keyboard {
    pub fn new(tx: SyncSender<RtMessage>) -> Self {
        Self { tx, frequency_ports: HashSet::new(), gate_ports: HashSet::new() }
    }

    pub fn run(&self) {
        use std::io::{self, BufRead};

        for line in io::stdin().lock().lines().flatten() {
            info!("Keyboard input: {}", line.trim());
            if line.trim() == "a" {
                for module in &self.frequency_ports {
                    let _ = self.tx.try_send(RtMessage::new(module.clone(), Frequency(440.0)));
                }
                // Open gates.
                for module in &self.gate_ports {
                    let _ = self.tx.try_send(RtMessage::new(module.clone(), Gate(true)));
                }
            }
            if line.trim() == "" {
                // Close gates.
                for module in &self.gate_ports {
                    let _ = self.tx.try_send(RtMessage::new(module.clone(), Gate(false)));
                }
            }
        }
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
}