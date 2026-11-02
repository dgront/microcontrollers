use std::f64::consts::TAU;

use crate::envelope::ADSREnvelope;
use crate::filters::Filter;
use crate::notes::Note;
use crate::settings::{BLOCK, DT};


/// Calculates the frequency of a note given a base frequency, an octave ratio, and a detune fraction in semitones.
pub fn calculate_frequency(f_base: f64, octave_ratio: f64, detune_fraction: f64) -> f64 {

    // Detuning is specified in fractions of a semitone, where 12 semitones make an octave.
    let detune_factor = 2.0f64.powf(detune_fraction / 12.0);

    f_base * octave_ratio * detune_factor
}

#[derive(Debug, Clone)]
pub enum SoundWave {
    Sine { ratio: f64, detune_fract: f64 },
    SineSqr { ratio: f64, detune_fract: f64 },
    Sawtooth { ratio: f64, detune_fract: f64 },
    Square { ratio: f64, detune_fract: f64 },
    Pulse { ratio: f64, duty: f64, detune_fract: f64 },
    Mix { waves: Vec<SoundWave> },
}

/// Implementations for a few sounds.
impl SoundWave {

    pub fn sine(ratio: f64) -> Self { Self::Sine { ratio, detune_fract: 0.0 } }

    pub fn sine_sqr(ratio: f64) -> Self { Self::SineSqr { ratio, detune_fract: 0.0 } }

    pub fn sawtooth(ratio: f64) -> Self { Self::Sawtooth { ratio, detune_fract: 0.0 } }

    pub fn square(ratio: f64) -> Self { Self::Square { ratio, detune_fract: 0.0 } }

    pub fn pulse(ratio: f64, duty: f64) -> Self { Self::Pulse { ratio, duty, detune_fract: 0.0 } }

    pub fn unison(base_sound: SoundWave, n_voices: u8, detune_fraction: f64) -> Self {
        // For unisono, if we have only one voice or no detune, we return the base sound directly.
        if n_voices <= 1 || detune_fraction == 0.0 { return base_sound; }

        let mut unison_waves = Vec::with_capacity(n_voices as usize);

        // For each voice, we calculate a detune offset based on its index and the total number of voices.
        match base_sound {
            // If the base sound is a Mix, we recursively apply unison to each wave in the mix.
            SoundWave::Mix { waves } => {
                for wave in waves {
                    let sub_unison = SoundWave::unison(wave, n_voices, detune_fraction);
                    if let SoundWave::Mix { waves: sub_waves } = sub_unison {
                        unison_waves.extend(sub_waves);
                    } else {
                        unison_waves.push(sub_unison);
                    }
                }
                return SoundWave::Mix { waves: unison_waves };
            }
            _ => {
                for i in 0..n_voices {
                    let spread_factor = if n_voices > 1 {
                        (i as f64 / (n_voices - 1) as f64) * 2.0 - 1.0
                    } else {
                        0.0
                    };

                    // Detune offset for this voice, scaled by the spread factor
                    let voice_detune_offset = spread_factor * detune_fraction;

                    // Deep copy of the base sound with adjusted detune for this voice
                    let voice_wave = match base_sound {
                        SoundWave::Sine { ratio, detune_fract } => SoundWave::Sine {
                            ratio,
                            detune_fract: detune_fract + voice_detune_offset,
                        },
                        SoundWave::SineSqr { ratio, detune_fract } => SoundWave::SineSqr {
                            ratio,
                            detune_fract: detune_fract + voice_detune_offset,
                        },
                        SoundWave::Sawtooth { ratio, detune_fract } => SoundWave::Sawtooth {
                            ratio,
                            detune_fract: detune_fract + voice_detune_offset,
                        },
                        SoundWave::Square { ratio, detune_fract } => SoundWave::Square {
                            ratio,
                            detune_fract: detune_fract + voice_detune_offset,
                        },
                        SoundWave::Pulse { ratio, duty, detune_fract } => SoundWave::Pulse {
                            ratio,
                            duty,
                            detune_fract: detune_fract + voice_detune_offset,
                        },
                        SoundWave::Mix { .. } => unreachable!(), // Dispatch for Mix is handled above, so we should never reach here.
                    };

                    unison_waves.push(voice_wave); // Each voice contributes equally to the mix
                }
            }
        }

        SoundWave::Mix { waves: unison_waves }

    }

    /// Renders a block of sound samples into the given output buffer.
    ///
    /// Sound sampling starts at `start` seconds, with a time step of `settings::DT` seconds between samples.
    /// The number of samples is determined by the length of the `out` buffer.
    /// Returns the time of the next sample after the last one.
    fn sample(&self, start: f64, freq: f64, out: &mut [f32]) -> f64 {

        match self {
            Self::SineSqr { ratio, detune_fract } => {
                let frequency = calculate_frequency(freq, *ratio, *detune_fract);
                let mut phase = (TAU * frequency * start).rem_euclid(TAU);
                let step = TAU * frequency * DT;

                for sample in out.iter_mut() {
                    *sample = phase.sin() as f32;
                    phase = (phase + step).rem_euclid(TAU);
                }
            }
            Self::Sine { ratio, detune_fract } => {
                let frequency = calculate_frequency(freq, *ratio, *detune_fract);
                let mut phase = (TAU * frequency * start).rem_euclid(TAU);
                let step = TAU * frequency * DT;

                for sample in out.iter_mut() {
                    *sample = phase.sin() as f32;
                    phase = (phase + step).rem_euclid(TAU);
                }
            }

            Self::Sawtooth { ratio, detune_fract } => {
                let frequency = calculate_frequency(freq, *ratio, *detune_fract);
                let step = frequency * DT;
                let mut phase = (frequency * start).rem_euclid(1.0);

                for sample in out.iter_mut() {
                    *sample = (2.0 * phase - 1.0) as f32;
                    phase = (phase + step).rem_euclid(1.0);
                }
            }

            Self::Square { ratio, detune_fract } => {
                let frequency = calculate_frequency(freq, *ratio, *detune_fract);
                let step = frequency * DT;
                let mut phase = (frequency * start).rem_euclid(1.0);

                for sample in out.iter_mut() {
                    *sample = if phase < 0.5 {
                        1.0
                    } else {
                        -1.0
                    };

                    phase = (phase + step).rem_euclid(1.0);
                }
            }
            SoundWave::Pulse { ratio, duty, detune_fract } => {
                let frequency = calculate_frequency(freq, *ratio, *detune_fract);
                let step = frequency * DT;
                let mut phase = (frequency * start).rem_euclid(1.0);

                for sample in out.iter_mut() {
                    *sample = if phase < *duty {
                        1.0
                    } else {
                        -1.0
                    };

                    phase = (phase + step).rem_euclid(1.0);
                }
            }
            Self::Mix { waves } => {
                // Clear the output buffer before mixing
                for sample in out.iter_mut() {
                    *sample = 0.0;
                }
                let mut temp_buffer = [0.0_f32; BLOCK];
                // For each wave in the mix, sample it and add it to the output buffer
                for wave in waves {
                    wave.sample(start, freq, &mut temp_buffer);
                    for i in 0..out.len() {
                        out[i] += temp_buffer[i];
                    }
                }
                for sample in out.iter_mut() {
                    *sample /= waves.len() as f32; // Normalize the mixed output
                }
            }
        }

        return start + DT * (out.len() as f64);
    }
}

#[derive(Debug, Clone)]
pub struct SoundComponent {
    pub wave: SoundWave,
    pub envelope: ADSREnvelope,
    buffer: [f32; BLOCK],
}

impl SoundComponent {
    pub fn new(wave: SoundWave, envelope: ADSREnvelope) -> Self {
        Self { wave, envelope, buffer: [0.0; BLOCK]}
    }

    pub fn sample(&mut self, start: f64, freq: f64) -> &mut [f32] {

        // --- Set the wave function value, in the range [-1.0, 1.0], into the buffer
        self.wave.sample(start, freq, &mut self.buffer);
        // --- Apply the envelope to the buffer, modifying it in place, i.e. multiplying the wave value by the envelope value at each sample
        self.envelope.apply(&mut self.buffer);

        return &mut self.buffer
    }

    pub fn release(&mut self) {
        self.envelope.release();
    }

    pub fn is_finished(&self) -> bool {
        self.envelope.current_state() == &crate::envelope::EnvelopeState::Iddle
    }
}


#[derive(Debug, Clone)]
pub struct Instrument {
    pub components: Vec<(f32, SoundComponent)>,
}

impl Instrument {
    pub fn single_component(component: SoundComponent) -> Self {
        Self { components: vec![(1.0, component)]}
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (f32, SoundComponent)> {
        self.components.iter_mut()
    }

    pub fn release(&mut self) {
        for (_, component) in self.components.iter_mut() {
            component.release();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sound {
    pub note: Note,
    pub octave: u16,
    pub timer: f64,
    pub filters: Vec<Filter>,
    instrument: Instrument,
}

impl Sound {

    pub fn new(note: Note, octave: u16, components: Instrument) -> Self {
        Self { note, octave, timer: 0.0, filters: vec![], instrument: components }
    }

    pub fn frequency(&self) -> f64 {
        self.note.octave_frequency(self.octave)
    }

    /// Samples the sound adding it into the given output buffer, applying the given factor to the amplitude.
    ///
    /// This method does not clear the output buffer, it just adds the sound to it.
    pub fn sample(&mut self, factor: f32, samples: &mut [i16]) {

        let freq = self.frequency();
        for (weight, component) in self.instrument.iter_mut() {
            let mut block = component.sample(self.timer, freq);
            for filter in self.filters.iter_mut() {
                filter.apply(&mut block);
            }
            for i in 0..samples.len() {
                samples[i] += (*weight * factor * block[i]) as i16;
            }
        }
        self.timer += DT * (samples.len() as f64);
    }

    pub fn release(&mut self) {
        self.instrument.release();
    }

    pub fn is_finished(&self) -> bool {
        self.instrument.components.iter().all(|(_, component)| component.is_finished())
    }
}
