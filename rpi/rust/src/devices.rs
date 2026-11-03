use crate::settings::*;

use std::error::Error;

pub type AudioResult<T> = Result<T, Box<dyn Error>>;

// ============================================================
// macOS: PortAudio
// ============================================================

#[cfg(target_os = "macos")]
mod mac {
    use super::*;
    use portaudio as pa;

    pub struct MacDevice {
        _pa: pa::PortAudio,
        stream: pa::Stream<pa::Blocking<pa::stream::Buffer>, pa::Output<f32>>,
    }

    impl MacDevice {
        pub fn new() -> AudioResult<Self> {
            let pa = pa::PortAudio::new()?;

            let settings = pa.default_output_stream_settings::<f32>(CHANNELS as i32, SAMPLING_RATE, BUFFER_FRAMES as u32)?;

            let mut stream = pa.open_blocking_stream(settings)?;
            stream.start()?;

            eprintln!("PortAudio device: {} Hz, {} channels, {} frames", SAMPLING_RATE, CHANNELS, BUFFER_FRAMES);

            Ok(Self { _pa: pa, stream })
        }

        pub fn write(&mut self, samples: &[f32]) -> AudioResult<()> {

            assert_eq!(samples.len() % CHANNELS, 0, "Audio buffer must contain complete frames");

            let n_frames = samples.len() / CHANNELS;

            if n_frames > 0 {
                self.stream.write(n_frames as u32, |output| { output.copy_from_slice(samples); })?;
            }

            Ok(())
        }
    }
}

// ============================================================
// Linux / Raspberry Pi: ALSA
// ============================================================

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use alsa::pcm::{Access, Format, HwParams, PCM};
    use alsa::{Direction, ValueOr};

    pub struct RPiDevice {
        pcm: PCM,
        output: [i32; BUFFER_FRAMES * CHANNELS],
    }

    impl RPiDevice {
        pub fn new() -> AudioResult<Self> {
            let pcm = PCM::new(ALSA_DEVICE, Direction::Playback, false)?;

            {
                let hwp = HwParams::any(&pcm)?;

                hwp.set_access(Access::RWInterleaved)?;
                hwp.set_format(Format::s32())?;
                hwp.set_channels(CHANNELS as u32)?;

                // Require the exact sampling rate used by the synth.
                hwp.set_rate(SAMPLING_RATE as u32, ValueOr::Nearest)?;

                let period = hwp.set_period_size_near(ALSA_PERIOD_FRAMES as i32, ValueOr::Nearest)?;

                let buffer = hwp.set_buffer_size_near(ALSA_BUFFER_FRAMES as i32)?;

                pcm.hw_params(&hwp)?;

                let actual_rate = pcm.hw_params_current()?.get_rate()?;
                if actual_rate != SAMPLING_RATE as u32 {
                    return Err(format!("ALSA selected {} Hz, expected {} Hz", actual_rate, SAMPLING_RATE).into());
                }

                eprintln!("ALSA: {} Hz, {} channels, period {}, buffer {} frames", actual_rate, CHANNELS, period, buffer);
            }

            pcm.prepare()?;

            Ok(Self { pcm, output: [0; BUFFER_FRAMES * CHANNELS] })
        }

        pub fn write(&mut self, samples: &[f32]) -> AudioResult<()> {
            assert_eq!(samples.len() % CHANNELS, 0, "Audio buffer must contain complete frames");
	    assert_eq!(samples.len(), self.output.len());

	    for (src, dst) in samples.iter().zip(self.output.iter_mut()) {
	        *dst = (src.clamp(-1.0, 1.0) as f64 * i32::MAX as f64) as i32;
	    }

            let io = self.pcm.io_i32()?;
            let frames = self.output.len() / CHANNELS;
            let mut written = 0;

            while written < frames {

                match io.writei(&self.output[written * CHANNELS..]) {
                    Ok(0) => {
                        return Err("ALSA wrote zero frames".into());
                    }
                    Ok(n) => {
                        written += n;
                    }
                    Err(err) => {
                        self.pcm.recover(err.errno(), true)?;
                    }
                }
            }

            Ok(())
        }
    }
}

// ============================================================
// Select backend at compile time
// ============================================================

#[cfg(target_os = "macos")]
pub use mac::MacDevice as Device;

#[cfg(target_os = "linux")]
pub use linux::RPiDevice as Device;