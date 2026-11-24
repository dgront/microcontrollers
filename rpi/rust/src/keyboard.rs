use std::collections::HashSet;
use std::sync::mpsc::SyncSender;
use std::str::FromStr;
use log::{debug, info};

use crate::modules::{RtMessage};
use crate::modules::Parameter::{Frequency, Gate, Waveform};
use crate::notes::Note;
use crate::sound::SoundWave;

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

    pub fn run(&mut self) -> Result<(), String> {
        use std::io::{self, BufRead};

        for line in io::stdin().lock().lines() {

            let Ok(line) = line else { break; };
            let input = line.trim();
            debug!("Keyboard input: {}", &input);

            match input {
                "+" => {
                    self.octave = self.octave.saturating_add(1);
                    info!("Octave: {}", self.octave);
                }

                "-" => {
                    self.octave = self.octave.saturating_sub(1);
                    info!("Octave: {}", self.octave);
                }
                "w" => {
                    self.send_waveform(SoundWave::Sawtooth);
                }

                "s" => {
                    self.send_waveform(SoundWave::Square);
                }

                "t" => {
                    self.send_waveform(SoundWave::Triangle);
                }

                "i" => {
                    self.send_waveform(SoundWave::Sine);
                }
                "" => {     // Close gates.
                    for module in &self.gate_ports {
                        let _ = self.tx.try_send(RtMessage::new(module.clone(), Gate(false)));
                    }
                }
                _ => {
                    match Note::from_str(input) {
                        Ok(note) => {
                            self.send_note_on(note.octave_frequency(self.octave));
                        }

                        Err(_) => {
                            eprintln!("Invalid keyboard input: {input}");
                        }
                    }
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
            let ret = self.tx.try_send(RtMessage::new(module.clone(), Frequency(frequency)));
            if let Err(err) = ret {
                log::error!("Keyboard failed to send message: {err}");
            }
        }
        for module in &self.gate_ports {
            let ret = self.tx.try_send(RtMessage::new(module.clone(), Gate(true)));
            if let Err(err) = ret {
                log::error!("Keyboard failed to send message: {err}");
            }
        }
    }

    fn send_waveform(&self, wave_type: SoundWave) {
        for module in &self.frequency_ports {
            let _ = self.tx.try_send(RtMessage::new(module.clone(), Waveform(wave_type)));
        }
    }
}