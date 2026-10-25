use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};
use std::f32::consts::PI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const RATE: u32 = 48_000;
    const FREQ: f32 = 440.0;
    const DURATION: usize = 5;
    const NOTE_DURATION: f32 = 0.5;
    const BLOCK: usize = 1024;
    // A natural minor scale: A B C D E F G A
    let notes = [
        440.00, 493.88, 523.25, 587.33,
        659.25, 698.46, 783.99, 880.00,
    ];

    // Open PCM5102A
    let pcm = PCM::new("plughw:1,0", Direction::Playback, false)?;

    // Configure ALSA
    let hwp = HwParams::any(&pcm)?;
    hwp.set_channels(2)?;
    hwp.set_rate(RATE, ValueOr::Nearest)?;
    hwp.set_format(Format::s16())?;
    hwp.set_access(Access::RWInterleaved)?;
    pcm.hw_params(&hwp)?;
    drop(hwp);

    // Generate 440 Hz stereo sine wave
    let mut samples = Vec::<i16>::new();

    for i in 0..RATE as usize * DURATION {
        let t = i as f32 / RATE as f32;
        let s = (10000.0 * (2.0 * PI * FREQ * t).sin()) as i16;
        samples.extend_from_slice(&[s, s]);
    }

    // Send samples to DAC
    let io = pcm.io_i16()?;
    let mut offset = 0;

    while offset < samples.len() / 2 {
        offset += io.writei(&samples[offset * 2..])?;
    }

    pcm.drain()?;
    Ok(())
}

