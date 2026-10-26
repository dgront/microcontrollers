mod devices;
mod settings;
mod notes;
mod sounds;

#[cfg(target_os = "linux")]
use alsa::pcm::{Access, Format, HwParams, PCM};
#[cfg(target_os = "linux")]
use alsa::{Direction, ValueOr};

use clap::{ArgGroup, Parser};

use crate::sounds::Sound;
use crate::notes::Notes;

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
    sin_sqr: bool,

    #[arg(long)]
    sawtooth: bool,

    #[arg(long)]
    square: bool,

    #[arg(long)]
    piano: bool,

    /// Notes per minute
    #[arg(long, default_value_t = 120.0)]
    npm: f32,
}


impl Args {
    fn instrument(&self) -> Sound {
        if self.sin_sqr {
            Sound::sine_sqr(1.0)
        } else if self.sin {
            Sound::sine(1.0)
        } else if self.sawtooth {
            Sound::sawtooth(1.0)
        } else if self.square {
            Sound::square(1.0)
        } else if self.piano {
            Sound::piano(1.0)
        } else {
            Sound::sine(1.0) // default
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
    let mut notes = Notes::new(3);
    // let a_minor = notes.frequencies(Notes::A_MINOR);
    // let mut notes_to_play = a_minor.clone();
    // notes_to_play.extend(a_minor.iter().rev().copied());
    // notes_to_play.extend(Notes::thirds(&a_minor));
    // notes_to_play.extend(Notes::thirds(&a_minor).iter().rev());

    let notes_to_play = vec![notes.frequency("A")];

    // Select instrument
    let instrument = args.instrument();
    let n_blocks: usize = args.blocks_per_note();
    eprintln!("Playing {} notes, {} blocks per note", notes_to_play.len(), n_blocks);

    let mut samples = [0_f32; settings::BLOCK * 2];
    let mut signals = [0_i16; settings::BLOCK * 2];
    let dt = 1.0 / settings::RATE as f64;
    let io = pcm.io_i16()?;
    for freq in notes_to_play {
        let mut time = 0.0_f64;
        for _ in 0..n_blocks {
            // Generate one block
            time = instrument.sample(time, dt, freq, &mut samples);
            for j in 0..samples.len() {
                signals[j] = (samples[j] * 10000.0) as i16;
            }

            // Send the block to a device
            let mut written = 0;
            while written < settings::BLOCK {
                let n = io.writei(&signals[written * 2..])?;
                if n == 0 { return Err("ALSA writei() returned 0 frames".into()); }
                written += n;
            }
        }
    }
    pcm.drain()?;
    Ok(())
}

