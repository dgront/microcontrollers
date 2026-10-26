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


const SOFT_PIANO: [f32; 10] =
    [1.0, 0.40, 0.25, 0.12, 0.08, 0.05, 0.03, 0.02, 0.01, 0.0];

fn piano(phase: f32) -> f32 { 

    let sum: f32 = SOFT_PIANO.iter().map(|c| c.abs()).sum();
    SOFT_PIANO
        .iter()
        .enumerate()
        .map(|(i, &c)| c * (phase * (i + 1) as f32).sin())
        .sum::<f32>() / sum
}

use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[command(version, about = "PCM5102A waveform synthesizer")]
#[command(group(
    ArgGroup::new("waveform")
        .args(["sin", "sin_abs", "sin_sqr", "sawtooth", "rectangular", "piano"])
        .multiple(false)
))]

struct Args {
    #[arg(long)]
    sin: bool,

    #[arg(long)]
    sin_abs: bool,

    #[arg(long)]
    sin_sqr: bool,

    #[arg(long)]
    sawtooth: bool,

    #[arg(long)]
    rectangular: bool,

    #[arg(long)]
    piano: bool,

    /// Notes per minute
    #[arg(long, default_value_t = 120.0)]
    npm: f32,
}


impl Args {
    fn waveform(&self) -> fn(f32) -> f32 {
        if self.sin_sqr {
            sin_sqr
        } else if self.sawtooth {
            sawtooth
        } else if self.rectangular {
            rectangular
        } else if self.piano {
            piano
        } else {
            sin // default
        }
    }

    fn blocks_per_note(&self) -> usize {
        return (60.0 * settings::RATE as f64 / (self.npm as f64 * settings::BLOCK as f64))
                .round()
                .max(1.0) as usize;
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::parse();

    // Open PCM5102A
    let pcm = devices::initialize_pcm()?;

    // what to play
    let mut notes = Notes::new(4);
    let a_minor = notes.frequencies(Notes::A_MINOR);
    // let mut notes_to_play = a_minor.clone();
    // notes_to_play.extend(a_minor.iter().rev().copied());
    // notes_to_play.extend(Notes::thirds(&a_minor));
    // notes_to_play.extend(Notes::thirds(&a_minor).iter().rev());
    let notes_to_play = vec![notes.frequency("A")];
    // Wave function
    let wave = args.waveform();

    let n_blocks: usize = args.blocks_per_note();
    
    let mut samples = [0_i16; settings::BLOCK * 2];
    let mut phase = 0.0_f32;
    let io = pcm.io_i16()?;
    for freq in notes_to_play {
        let phase_step = 2.0 * PI * freq / settings::RATE as f32;
        for _ in 0..n_blocks {
            // Generate one block
            for frame in samples.chunks_exact_mut(2) {
               let value_l = (wave(phase) * 10000.0) as i16;

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

