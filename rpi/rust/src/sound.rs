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

fn sin(phase: f32) -> f32 { phase.sin() }

fn sin_abs(phase: f32) -> f32 { phase.sin().abs() }

fn sin_sqr(phase: f32) -> f32 { 
    let v = phase.sin().abs();
    return v*v;
}

fn sawtooth(phase: f32) -> f32 {
    return (phase / (2.0*PI)).fract();
}

fn rectangular(phase: f32) -> f32 {
    if phase > PI  { return 1.0; } else { return -1.0; }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {


    const BLOCKS_PER_NOTE: usize = 24;

    // A natural minor scale: A B C D E F G A
    // let notes = [
    //     440.00, 493.88, 523.25, 587.33,
    //     659.25, 698.46, 783.99, 880.00,
    // ];


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
//               let value_l = (sin(phase) * 10000.0) as i16;
               let value_l = (sin_sqr(phase) * 10000.0) as i16;
//                let value_l = (sawtooth(phase) * 10000.0) as i16;
//                let value_l = (rectangular(phase) * 10000.0) as i16;
                //                let value_r = (sawtooth(phase, freq, RATE as f32) * 10000.0) as i16;
                //                let value_r = (rectangular(phase, freq, RATE as f32) * 10000.0) as i16;

                frame[0] = value_l; // left
                frame[1] = value_l; // right

                phase = (phase + phase_step) % (2.0 * PI);
            }

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

