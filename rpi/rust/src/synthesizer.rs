use std::collections::HashMap;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use crate::devices::{AudioResult, Device};
use crate::keyboard::Keyboard;
use crate::modules::{Input, Module, RtMessage};
use crate::settings::{BUFFER_FRAMES, CHANNELS};

pub struct PipelineStep {
    pub module_id: String,
    pub inputs: HashMap<Input, usize>,
    pub output: usize,
}

pub struct Synthesizer {
    tx: SyncSender<RtMessage>,
    rx: Receiver<RtMessage>,
    modules: HashMap<String, Box<dyn Module>>,
    device: Device,
    buffers: Vec<[f32; BUFFER_FRAMES]>,
    pipeline: Vec<PipelineStep>
}

impl Synthesizer {
    pub fn new(device: Device) -> Self {
        let (tx, rx) = sync_channel(1024);
        let modules: HashMap<String, Box<dyn Module>> = HashMap::new();
        Self { tx, rx, modules, device, buffers: Vec::new(), pipeline: Vec::new() }
    }

    pub fn add_module(&mut self, module: Box<dyn Module>) {
        let name = module.name().to_string();
        self.modules.insert(name, module);
    }

    pub fn add_connection(&mut self, module_name: &str, inputs: HashMap<Input, usize>, output: usize) {

        if !self.modules.contains_key(module_name) {
            eprintln!("Module {} not found", module_name);
            return;
        }

        // ---------- Ensure all referenced buffers exist.
        let max_index = inputs.values().copied().chain(std::iter::once(output)).max().unwrap();
        // ---------- Create a new buffer if necessary. Initialize with zeros.
        self.buffers.resize(max_index + 1, [0.0; BUFFER_FRAMES]);
        // ---------- Add the module to the processing pipeline.
        self.pipeline.push(PipelineStep { module_id: module_name.to_string(), inputs, output});
    }

    pub fn sender(&self) -> SyncSender<RtMessage> {
        self.tx.clone()
    }

    /// Send a message to a module in the synthesizer.
    pub fn send_message(&self, message: RtMessage) {
        let _ = self.tx.send(message);
    }

    pub fn run(&mut self) -> AudioResult<()> {

        // let keyboard = Keyboard::new(self.tx.clone());
        //
        // // Spawn a separate thread to run the keyboard input loop.
        // std::thread::spawn(move || keyboard.run());

        let mut buffer = [0.0f32; BUFFER_FRAMES * CHANNELS];

        loop {
            while let Ok(message) = self.rx.try_recv() {
                self.handle_message(message);
            }

            // Execute the synthesis pipeline.
            for step in &self.pipeline {
                let output_idx = step.output;

                // Split buffers to borrow the output mutably and earlier input buffers immutably.
                let (before, output_and_after) =
                    self.buffers.split_at_mut(output_idx);

                let output = &mut output_and_after[0];

                let inputs: HashMap<Input, &[f32]> = step.inputs
                    .iter()
                    .map(|(&input_type, &buffer_idx)| {
                        (
                            input_type,
                            &before[buffer_idx][..],
                        )
                    })
                    .collect();

                let module = self.modules
                    .get_mut(&step.module_id)
                    .expect("Unknown module in pipeline");

                module.sample(inputs, output);
            }

            // Use the output of the last pipeline step.
            let final_idx = self.pipeline.last().unwrap().output;
            let final_buffer = &self.buffers[final_idx];

            // Convert mono to interleaved audio channels.
            for (frame, &sample) in final_buffer.iter().enumerate() {
                for channel in 0..CHANNELS {
                    buffer[frame * CHANNELS + channel] = sample;
                }
            }

            self.device.write(&buffer)?;
        }
    }

    fn handle_message(&mut self, message: RtMessage) {
        if let Some(module) = self.modules.get_mut(&message.receiver_module) {
            module.set_parameter(message.value);
        }
    }
}