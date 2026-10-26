mod devices;
mod settings;
mod notes;

#[cfg(target_os = "linux")]
use alsa::pcm::{Access, Format, HwParams, PCM};
#[cfg(target_os = "linux")]
use alsa::{Direction, ValueOr};
use std::f32::consts::PI;

use crate::notes::Notes;

const RATE: u32 = 48_000;


fn sawtooth(phase: f32, frequency: f32, rate: f32) -> f32 {
    let samples_per_period = rate / frequency;
    return (phase / samples_per_period).fract();
}

fn rectangular(phase: f32, frequency: f32, rate: f32) -> f32 {
    let samples_per_period = rate / frequency;
    let step = ((phase * 2.0)).trunc() as i32;
    println!("{} {}", phase, step);
    if step % 2 == 0 { return 1.4; } else { return 0.2; }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {


    const BLOCKS_PER_NOTE: usize = 24;
    
    // Open PCM5102A
    let pcm = devices::initialize_pcm()?;
    let mut notes = Notes::new(4);
    let a_minor = notes.frequencies(Notes::A_MINOR);
    let mut up_down = a_minor.clone();
    up_down.extend(a_minor.iter().rev().copied());
    up_down.extend(Notes::thirds(&a_minor));
    up_down.extend(Notes::thirds(&a_minor).iter().rev());

    let mut samples = [0_i16; settings::BLOCK * 2];
    let mut phase = 0.0_f32;
    let io = pcm.io_i16()?;
    for freq in up_down {
        let phase_step = 2.0 * PI * freq / settings::RATE as f32;
        for _ in 0..BLOCKS_PER_NOTE {
            // Generate one block
            for frame in samples.chunks_exact_mut(2) {
                let value_l = ((phase).sin() * 10000.0) as i16;
                //                let value_r = (sawtooth(phase, freq, RATE as f32) * 10000.0) as i16;
                //                let value_r = (rectangular(phase, freq, RATE as f32) * 10000.0) as i16;

                frame[0] = value_l; // left
                frame[1] = value_l; // right

                phase = (phase + phase_step) % (2.0 * PI);
            }
            println!("last phase: {} for note {}", phase, freq);

            // Send the block to ALSA
            let mut written = 0;
            while written < settings::BLOCK {
                let n = io.writei(&samples[written * 2..])?;
                if n == 0 { return Err("ALSA writei() returned 0 frames".into()); }
                written += n;
            }
        }
    }
    pcm.drain()?;
    Ok(())
}

