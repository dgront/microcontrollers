mod devices;

#[cfg(target_os = "linux")]
use alsa::pcm::{Access, Format, HwParams, PCM};
#[cfg(target_os = "linux")]
use alsa::{Direction, ValueOr};
use std::f32::consts::PI;

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
    const BLOCK: usize = 1024;

    // A natural minor scale: A B C D E F G A
    let notes = [
        440.00, 493.88, 523.25, 587.33,
        659.25, 698.46, 783.99, 880.00,
    ];

    // Open PCM5102A
    let pcm = devices::initialize_pcm()?;


    // Generate 440 Hz stereo sine wave
    //    let mut samples = Vec::<i16>::new();
    let mut samples = [0_i16; BLOCK * 2];
    let mut phase = 0.0_f32;
    let io = pcm.io_i16()?;
    for freq in notes {
        let phase_step = 2.0 * PI * freq / RATE as f32;
        for _ in 0..BLOCKS_PER_NOTE {
            // Generate one block
            println!("n_frames: {:?}", samples.chunks_exact_mut(2));
            for frame in samples.chunks_exact_mut(2) {
                let value_l = ((phase).sin() * 10000.0) as i16;
                //                let value_r = (sawtooth(phase, freq, RATE as f32) * 10000.0) as i16;
                //                let value_r = (rectangular(phase, freq, RATE as f32) * 10000.0) as i16;

                frame[0] = value_l; // left
                frame[1] = value_l; // right

                phase = (phase + phase_step) % (2.0 * PI);
            }
            println!("last phase: {}", phase);

            // Send the block to ALSA
            let mut written = 0;
            while written < BLOCK {
                let n = io.writei(&samples[written * 2..])?;
                if n == 0 { return Err("ALSA writei() returned 0 frames".into()); }
                written += n;
            }
        }
        pcm.drain()?;
    }
    Ok(())
}

