pub const SAMPLING_RATE: f64 = 48_000.0;
pub const CHANNELS: usize = 2;

pub const BUFFER_FRAMES: usize = 256;
pub const ALSA_PERIOD_FRAMES: usize = 128;
pub const ALSA_BUFFER_FRAMES: usize = 512;

pub const DEFAULT_FREQUENCY: f64 = 440.0;
pub const MASTER_VOLUME: f32 = 0.5;

#[cfg(target_os = "linux")]
pub const ALSA_DEVICE: &str = "plughw:1,0";