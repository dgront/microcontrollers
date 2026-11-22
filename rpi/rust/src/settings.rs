pub const SAMPLING_RATE: f64 = 48_000.0;
pub const DT: f64 = 1.0 / SAMPLING_RATE;

pub const CHANNELS: usize = 1;

pub const BUFFER_FRAMES: usize = 1024;
pub const PERIOD_FRAMES: usize = 256;

pub const DEFAULT_FREQUENCY: f64 = 440.0;
pub const MASTER_VOLUME: f64 = 0.5;

#[cfg(target_os = "linux")]
pub const ALSA_DEVICE: &str = "plughw:1,0";