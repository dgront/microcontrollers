use std::f64::consts::TAU;

#[derive(Clone, Copy)]
pub struct Adsr {
    pub attack: f64,
    pub decay: f64,
    pub sustain: f32,
    pub release: f64,
    pub duration: f64,
}

impl Adsr {
    pub fn value(&self, t: f64) -> f32 {

        let level = |t: f64| -> f32 {
            if self.attack > 0.0 && t < self.attack {
                (t / self.attack) as f32
            } else if self.decay > 0.0 && t < self.attack + self.decay {
                let x = (t - self.attack) / self.decay;
                1.0 + (self.sustain - 1.0) * x as f32
            } else {
                self.sustain
            }
        };

        if t < self.duration {
            level(t)
        } else if self.release > 0.0 && t < self.duration + self.release {
            let release_level = level(self.duration);
            let x = (t - self.duration) / self.release;

            release_level * (1.0 - x as f32)
        } else {
            0.0
        }
    }
}

pub enum Sound {
    Sine { ratio: f64 },
    SineSqr { ratio: f64 },
    Sawtooth { ratio: f64 },
    Square { ratio: f64 },
    Mix(Vec<(f32, Sound)>),
    Enveloped { source: Box<Sound>, adsr: Adsr},
}

/// Implementations for very basic sounds.
impl Sound {

    pub fn sine(ratio: f64) -> Self { Self::Sine { ratio } }

    pub fn sine_sqr(ratio: f64) -> Self { Self::SineSqr { ratio } }

    pub fn sawtooth(ratio: f64) -> Self { Self::Sawtooth { ratio } }

    pub fn square(ratio: f64) -> Self { Self::Square { ratio } }

    pub fn mix(components: impl IntoIterator<Item = (f32, Sound)>) -> Self {
        Self::Mix(components.into_iter().collect())
    }

    pub fn envelope(self, adsr: Adsr) -> Self {
        Self::Enveloped {
            source: Box::new(self),
            adsr,
        }
    }

    /// Returns the value of the sound of a given frequency `freq` at a given time (in seconds).
    pub fn value(&self, time: f64, freq: f64) -> f32 {
        match self {
            Self::Sine { ratio } => {
                (TAU * freq * ratio * time).sin() as f32
            }

            Self::SineSqr { ratio } => {
                let v = (TAU * freq * ratio * time).sin() as f32;
                v * v
            }

            Self::Sawtooth { ratio } => {
                (2.0 * (freq * ratio * time).rem_euclid(1.0) - 1.0) as f32
            }

            Self::Square { ratio } => {
                if (freq * ratio * time).rem_euclid(1.0) < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }

            Self::Mix(components) => {
                components.iter()
                    .map(|(weight, sound)| weight * sound.value(time, freq))
                    .sum()
            }

            Self::Enveloped { source, adsr } => {
                adsr.value(time) * source.value(time, freq)
            }
        }
    }

    /// Renders a block of sound samples into the given output buffer.
    ///
    /// Sound sampling starts at `start` seconds, with a time step of `dt` seconds between samples.
    /// The number of samples is determined by the length of the `out` buffer.
    /// Returns the time of the next sample after the last one.
    pub fn sample(&self, start: f64, dt: f64, freq: f64, out: &mut [f32]) -> f64 {
        out.fill(0.0);
        self.render_add(start, dt, freq, 1.0, &|_| 1.0, out);

        return start + dt * (out.len() as f64);
    }

    // Add weighted samples to an existing buffer.
    // This is the key operation for efficient composition.
    fn render_add(&self, start: f64, dt: f64, freq: f64, weight: f32, gain: &dyn Fn(f64) -> f32, out: &mut [f32]) {

        match self {
            Self::SineSqr { ratio } => {
                let frequency = freq * ratio;
                let mut phase = (TAU * frequency * start).rem_euclid(TAU);
                let step = TAU * frequency * dt;

                for sample in out.iter_mut() {
                    let v = phase.sin() as f32;
                    *sample += weight * v * v;
                    phase = (phase + step).rem_euclid(TAU);
                }
            }
            Self::Sine { ratio } => {
                let frequency = freq * ratio;
                let mut phase = (TAU * frequency * start).rem_euclid(TAU);
                let step = TAU * frequency * dt;

                for (i, sample) in out.iter_mut().enumerate() {
                    let t = start + i as f64 * dt;
                    *sample += weight * gain(t) * phase.sin() as f32;
                    phase = (phase + step).rem_euclid(TAU);
                }
            }

            Self::Sawtooth { ratio } => {
                let frequency = freq * ratio;
                let step = frequency * dt;
                let mut phase = (frequency * start).rem_euclid(1.0);

                for sample in out.iter_mut() {
                    *sample += weight * (2.0 * phase - 1.0) as f32;
                    phase = (phase + step).rem_euclid(1.0);
                }
            }

            Self::Square { ratio } => {
                let frequency = freq * ratio;
                let step = frequency * dt;
                let mut phase = (frequency * start).rem_euclid(1.0);

                for sample in out.iter_mut() {
                    *sample += weight * if phase < 0.5 {
                        1.0
                    } else {
                        -1.0
                    };

                    phase = (phase + step).rem_euclid(1.0);
                }
            }

            Self::Enveloped { source, adsr } => {
                source.render_add(start, dt, freq, weight, &|t| gain(t) * adsr.value(t), out);
            }

            Self::Mix(components) => {
                for (w, sound) in components {
                    sound.render_add(start, dt, freq, weight * w, gain, out);
                }
            }
        }
    }
}

impl Sound {
    pub fn piano(_ratio: f64) -> Self {
        const COEFF: [f32; 6] = [1.0, 0.40, 0.25, 0.12, 0.08, 0.05];
        // const COEFF: [f32; 10] = [1.0, 0.40, 0.25, 0.12, 0.08, 0.05, 0.03, 0.02, 0.01, 0.0];

        let norm: f32 = COEFF.iter().sum();

        Self::Mix(
            COEFF.iter()
                .enumerate()
                .filter(|(_, &c)| c > 0.0)
                .map(|(i, &c)| {
                    let harmonic = (i + 1) as f64;

                    let adsr = Adsr {
                        attack: 0.003,
                        decay: 0.15 / harmonic.sqrt(),
                        sustain: 0.25 / harmonic.sqrt() as f32,
                        release: 0.12 / harmonic.sqrt(),
                        duration: 0.5,
                    };

                    let partial = Self::Sine {
                        ratio: harmonic,
                    }
                        .envelope(adsr);

                    (c / norm, partial)
                })
                .collect()
        )
    }
}