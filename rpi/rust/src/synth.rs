mod devices;
mod settings;
mod notes;
mod sound_waves;
mod scales;
mod envelope;
mod interactive;

#[cfg(target_os = "linux")]
use alsa::pcm::{Access, Format, HwParams, PCM};
#[cfg(target_os = "linux")]
use alsa::{Direction, ValueOr};

#[cfg(not(target_os = "linux"))]
use crate::devices::MockupPCM as PCM;

use clap::{ArgGroup, Parser};
use crate::interactive::play_interactive;
use crate::sound_waves::{Instrument, Sound, SoundComponent, SoundWave};
use crate::notes::Note;

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

    /// pulse wave with given duty cycle (0.0 to 1.0)
    #[arg(long)]
    pulse: Option<f64>,

    #[arg(long)]
    square: bool,

    #[arg(long)]
    piano: bool,

    /// Notes per minute
    #[arg(long, default_value_t = 120.0)]
    npm: f32,

    /// Interactive mode
    #[arg(long)]
    interactive: bool,
}


impl Args {
    fn wave_type(&self) -> SoundWave {
        if self.sin_sqr {
            SoundWave::sine_sqr(1.0)
        } else if self.sin {
            SoundWave::sine(1.0)
        } else if self.sawtooth {
            SoundWave::sawtooth(1.0)
        } else if self.square {
            SoundWave::square(1.0)
        }  else if let Some(duty) = self.pulse {
            SoundWave::pulse(1.0, duty.clamp(0.0, 1.0))
        }    else {
            SoundWave::sine(1.0) // default
        }
    }

    fn blocks_per_note(&self) -> usize {
        return (60.0 * settings::RATE as f64 / (self.npm as f64 * settings::BLOCK as f64))
                .round()
                .max(1.0) as usize;
    }
}


fn play_notes(notes_to_play: &[Note], instrument: &Instrument, blocks_per_note: usize, pcm: &PCM) -> Result<(), Box<dyn std::error::Error>> {

    let mut mono = [0_i16; settings::BLOCK];
    let mut signals = [0_i16; settings::BLOCK * settings::CHANNELS];
    let io = pcm.io_i16()?;
    for note in notes_to_play {
        let mut sound = Sound::new(*note, 4, instrument.clone());
        for _ in 0..blocks_per_note {
            // Clear the buffer after each block
            mono.fill(0);
            // Generate one block of the mono sound
            sound.sample(10_000.0, &mut mono);
            // Convert mono to interleaved stereo
            for (src, dst) in mono.iter().zip(signals.chunks_exact_mut(2)) {
                dst[0] = *src;  // Left
                dst[1] = *src;  // Right
            }
            // Send the block to a device
            let mut written = 0;
            while written < settings::BLOCK {
                let n = io.writei(&signals[written * settings::CHANNELS..])?;
                if n == 0 { return Err("ALSA writei() returned 0 frames".into()); }
                written += n;
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::parse();

    // Open PCM5102A or its mockup
    let pcm = devices::initialize_pcm()?;

    // what to play
    let notes_to_play = vec![Note::A];

    // Select instrument
    let envelope = envelope::ADSREnvelope::new(0.05, 0.35, 0.5, 1.0, 0.3, 0.1);
    eprintln!("Using envelope:\n{}", envelope);
    let mut instrument
        = Instrument::single_component(SoundComponent::new(args.wave_type(), envelope));
    let n_blocks: usize = args.blocks_per_note();
    eprintln!("Playing {} notes, {} blocks per note", notes_to_play.len(), n_blocks);

    if args.interactive {
        return play_interactive(&pcm, instrument);
    } else {
        play_notes(&notes_to_play, &mut instrument, n_blocks, &pcm)?;
    }
    // pcm.drain()?;
    Ok(())
}

