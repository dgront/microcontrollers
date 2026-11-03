use std::f32::consts::PI;

#[cfg(target_os = "macos")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

struct SynthEngine {
    phase: f32,
    phase_inc: f32,
}

impl SynthEngine {
    fn new(sample_rate: f32, freq: f32) -> Self {
        Self {
            phase: 0.0,
            phase_inc: (2.0 * PI * freq) / sample_rate,
        }
    }

    fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for frame in buffer.chunks_exact_mut(2) {
            let sample = self.phase.sin();
            self.phase = (self.phase + self.phase_inc) % (2.0 * PI);
            frame[0] = sample; // Lewy
            frame[1] = sample; // Prawy
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sample_rate = 48000.0;
    let mut synth = SynthEngine::new(sample_rate, 440.0);

    #[cfg(target_os = "linux")]
    {
        println!("backend ALSA 0.9 (Raspberry Pi)...");
        run_alsa_backend(&mut synth, sample_rate as u32)?;
    }

    #[cfg(target_os = "macos")]
    {
        println!("backend CPAL (macOS)...");
        run_cpal_backend( synth, sample_rate)?;
    }

    Ok(())
}

// =========================================================================
#[cfg(target_os = "linux")]
fn run_alsa_backend(synth: &mut SynthEngine, sample_rate: u32) -> Result<(), Box<dyn std::error::Error>> {
    use alsa::pcm::{PCM, Format, Access, HwParams};
    use alsa::ValueOr;
    
    let pcm = PCM::new("plughw:1,0", alsa::Direction::Playback, false)?;
    
    let hwp = HwParams::any(&pcm)?;
    hwp.set_access(Access::RWInterleaved)?;      
    hwp.set_format(Format::s16())?; 
    hwp.set_channels(2)?;
    hwp.set_rate(sample_rate, ValueOr::Nearest)?;
        
    let actual_period_frames = hwp.set_period_size_near(128, ValueOr::Nearest)?; 
        
    let actual_buffer_frames = hwp.set_buffer_size_near(512)?;
        
    eprintln!("Wynegocjowane parametry sprzętowe: Period = {} ramek, Buffer = {} ramek", 
	actual_period_frames, actual_buffer_frames);

    pcm.hw_params(&hwp)?;
    pcm.prepare()?;
    
    let io = pcm.io_i16()?;
    
    let mut f32_buf = vec![0.0_f32; 256]; // 128 frames x 2 channels
    let mut i32_buf = vec![0_i16; 256];

    let max_val = 10_000.0_f32;

    println!("Syntezator (ALSA 0.9) działa w 16-bit. Ctrl+C aby przerwać.");

    loop {
        synth.fill_buffer(&mut f32_buf);

        for (f_sample, i_sample) in f32_buf.iter().zip(i32_buf.iter_mut()) {
            *i_sample = (f_sample.clamp(-1.0, 1.0) * max_val) as i16;
        }

        match io.writei(&i32_buf) {
            Ok(_) => {},
            Err(err) => {
eprintln!("Can't write to device: {:?}", err);
            }
        }
    }
}

// =========================================================================
#[cfg(target_os = "macos")]
fn run_cpal_backend(synth: SynthEngine, sample_rate: f32) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::{Arc, Mutex};

    let host = cpal::default_host();
    let device = host.default_output_device().expect("Nie znaleziono domyślnej karty na Macu");
    
    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: cpal::SampleRate(sample_rate as u32),
        buffer_size: cpal::BufferSize::Fixed(256), 
    };

    let synth_state = Arc::new(Mutex::new(synth));
    let synth_clone = Arc::clone(&synth_state);

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            if let Ok(mut s) = synth_clone.lock() {
                s.fill_buffer(data);
            }
        },

        |err| eprintln!("Błąd strumienia cpal: {}", err),
        None
    )?;

    stream.play()?;
    println!("Syntezator on MacOS (f32). Naciśnij Enter, aby zakończyć...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(())
}

