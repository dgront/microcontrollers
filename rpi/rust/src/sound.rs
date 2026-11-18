use std::f64::consts::TAU;
use crate::settings::DT;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Represents the type of waveform this synthesizer can generate.
pub enum SoundWave {
    Sine,
    Square,
    Triangle,
    Sawtooth,
}

impl SoundWave {
    /// Generate samples at a constant frequency.
    ///
    /// `phase` is the current phase of the waveform, which will be updated after generating samples.
    /// The result is stored in the `out` buffer.
    pub fn sample(&self, phase: &mut f64, frequency: f64, out: &mut [f32]) {
        self.render(phase, frequency, None, out);
    }

    /// Generate samples with additive frequency modulation in Hz.
    ///
    /// `phase` is the current phase of the waveform, which will be updated after generating samples.
    /// The result is stored in the `out` buffer.
    pub fn sample_modulated(&self, phase: &mut f64, frequency: f64, modulation: &[f32], out: &mut [f32]) {
        self.render(phase, frequency, Some(modulation), out);
    }

    /// Shared waveform dispatch.
    fn render(&self, phase: &mut f64, frequency: f64, modulation: Option<&[f32]>, out: &mut [f32]) {

        match self {
            SoundWave::Sine => {
                generate(phase, frequency, modulation, out, |p| {
                    (TAU * p).sin() as f32
                });
            }

            SoundWave::Square => {
                generate(phase, frequency, modulation, out, |p| {
                    if p < 0.5 { 1.0 } else { -1.0 }
                });
            }

            SoundWave::Triangle => {
                generate(phase, frequency, modulation, out, |p| {
                    (1.0 - 4.0 * (p - 0.5).abs()) as f32
                });
            }

            SoundWave::Sawtooth => {
                generate(phase, frequency, modulation, out, |p| {
                    (2.0 * p - 1.0) as f32
                });
            }
        }
    }
}

/// Shared sample generation loop.
///
/// Assumes instantaneous frequencies are nonnegative and lower than SAMPLING_FREQ.
#[inline]
fn generate<F: Fn(f64) -> f32>(phase: &mut f64, frequency: f64, modulation: Option<&[f32]>,
                               out: &mut [f32], waveform: F) {

    let mut p = *phase;

    match modulation {
        None => {
            let step = frequency * DT;

            for sample in out.iter_mut() {
                *sample = waveform(p);
                p += step;
                if p >= 1.0 { p -= 1.0; }
            }
        }

        Some(buffer) => {
            assert_eq!(buffer.len(), out.len(), "Modulation and output buffers must have equal lengths");
            for (sample, &m) in out.iter_mut().zip(buffer.iter()) {
                *sample = waveform(p);
                let instantaneous_frequency = voltage_to_frequency(frequency, m);
                p += instantaneous_frequency * DT;
                if p >= 1.0 { p -= 1.0; }
            }
        }
    }

    *phase = p;
}

#[inline]
fn voltage_to_frequency(base_frequency: f64, voltage: f32) -> f64 {
    const OCTAVES_PER_VOLT: f32 = 1.0;

    base_frequency * (voltage * OCTAVES_PER_VOLT).exp2() as f64
}