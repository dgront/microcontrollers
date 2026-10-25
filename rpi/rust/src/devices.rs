use std::io;

#[cfg(target_os = "linux")]
use alsa::pcm::{Access, Format, HwParams};
#[cfg(target_os = "linux")]
use alsa::pcm::PCM;

#[cfg(target_os = "linux")]
use alsa::{Direction, ValueOr};

use crate::settings::{RATE, BLOCK, CHANNELS};

pub struct MockupPCM;

pub struct MockupIO;


#[cfg(not(target_os = "linux"))]
use MockupPCM as PCM;

#[cfg(target_os = "linux")]
pub fn initialize_pcm() -> Result<PCM, Box<dyn std::error::Error>> {

    // Open PCM5102A
    let pcm = PCM::new("plughw:1,0", Direction::Playback, false)?;

    // Configure ALSA
    let hwp = HwParams::any(&pcm)?;
    hwp.set_channels(CHANNELS)?;
    hwp.set_rate(RATE, ValueOr::Nearest)?;
    hwp.set_format(Format::s16())?;
    hwp.set_access(Access::RWInterleaved)?;
    pcm.hw_params(&hwp)?;
    drop(hwp);

    Ok(pcm)
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn initialize_pcm() -> Result<PCM, Box<dyn std::error::Error>> {
    Ok(MockupPCM::new())
}

impl MockupPCM {
    pub fn new() -> Self { Self }

    pub fn io_i16(&self) -> io::Result<MockupIO> { Ok(MockupIO) }

    pub fn drain(&self) -> io::Result<()> { Ok(()) }
}

impl MockupIO {
    pub fn writei(&self, samples: &[i16]) -> io::Result<usize> {
        for frame in samples.chunks_exact(2) {
            println!("L: {}, R: {}", frame[0], frame[1]);
        }

        // ALSA writei() returns frames, not samples
        Ok(samples.len() / 2)
    }
}
