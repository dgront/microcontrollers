use std::sync::mpsc::SyncSender;
use log::info;
use crate::modules::{PortId, RtMessage};
use crate::modules::Parameter::{Frequency, Gate};

pub struct Keyboard {
    tx: SyncSender<RtMessage>,
    frequency_port: PortId,
}

impl Keyboard {
    pub fn new(tx: SyncSender<RtMessage>, frequency_port: PortId) -> Self {
        Self { tx, frequency_port }
    }

    pub fn run(&self) {
        use std::io::{self, BufRead};

        for line in io::stdin().lock().lines().flatten() {
            info!("Keyboard input: {}", line.trim());
            if line.trim() == "a" {
                let _ = self.tx.try_send(RtMessage {
                    port: self.frequency_port.clone(),
                    value: Frequency(440.0), // A4 frequency
                });
                let _ = self.tx.try_send(RtMessage {
                    port: self.frequency_port.clone(),
                    value: Gate(true), // open the gate
                });
            }
            if line.trim() == "" {
                let _ = self.tx.try_send(RtMessage {
                    port: self.frequency_port.clone(),
                    value: Gate(false), // shut the gate
                });
            }
        }
    }
}