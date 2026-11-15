
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use crate::devices::{AudioResult, Device};
use crate::keyboard::Keyboard;
use crate::modules::{Module, RtMessage};
use crate::modules::Parameter::{Frequency, Gate};
use crate::modules::Oscillator;
use crate::settings::{BUFFER_FRAMES, CHANNELS};

pub struct Synthesizer {
    tx: SyncSender<RtMessage>,
    rx: Receiver<RtMessage>,
    oscillator: Oscillator,
    device: Device,
    buffer: [f32; BUFFER_FRAMES * CHANNELS],
}

impl Synthesizer {
    pub fn new(device: Device) -> Self {
        let (tx, rx) = sync_channel(1024);

        let oscillator = Oscillator::new("Wave1".to_string());
        Self { tx, rx, oscillator, device, buffer: [0.0; BUFFER_FRAMES * CHANNELS ]  }
    }

    pub fn run(&mut self) -> AudioResult<()> {
        let keyboard = Keyboard::new(self.tx.clone(), self.oscillator.id());

        // Spawn a separate thread to run the keyboard input loop.
        std::thread::spawn(move || keyboard.run());

        let mut buffer = [0.0f32; BUFFER_FRAMES * CHANNELS];

        loop {
            while let Ok(message) = self.rx.try_recv() {
                self.handle_message(message);
            }

            // Generate the next audio block.
            self.oscillator.fill_buffer(&mut buffer);

            // Play the generated samples.
            self.device.write(&buffer)?;
        }
    }

    fn handle_message(&mut self, message: RtMessage) {
        match message.value {
            Frequency(freq) => {
                self.oscillator.set_parameter(Frequency(freq));
            }
            Gate(state) => {
                self.oscillator.set_parameter(Gate(state));
            }
            _ => {}
        }
    }
}