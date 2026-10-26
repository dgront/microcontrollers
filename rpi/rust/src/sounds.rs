use std::f64::consts::TAU;

pub enum Sound {
    Sine { ratio: f64 },
    SineSqr { ratio: f64 },
    Sawtooth { ratio: f64 },
    Square { ratio: f64 },
    Mix(Vec<(f32, Sound)>),
}

impl Sound {

    pub fn sine(ratio: f64) -> Self { Self::Sine { ratio } }

    pub fn sine_sqr(ratio: f64) -> Self { Self::SineSqr { ratio } }

    pub fn sawtooth(ratio: f64) -> Self { Self::Sawtooth { ratio } }

    pub fn square(ratio: f64) -> Self { Self::Square { ratio } }

    pub fn mix(components: impl IntoIterator<Item = (f32, Sound)>) -> Self {
        Self::Mix(components.into_iter().collect())
    }

    pub fn piano(_ratio: f64) -> Self {
        const COEFF: [f32; 10] = [1.0, 0.40, 0.25, 0.12, 0.08, 0.05, 0.03, 0.02, 0.01, 0.0];

        let norm: f32 = COEFF.iter().map(|c| c.abs()).sum();

        Self::mix(
            COEFF.iter()
                .enumerate()
                .map(|(i, &c)| {
                    (c / norm, Self::sine((i + 1)  as f64))
                })
        )
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
        }
    }

    /// Renders a block of sound samples into the given output buffer.
    ///
    /// Sound sampling starts at `start` seconds, with a time step of `dt` seconds between samples.
    /// The number of samples is determined by the length of the `out` buffer.
    /// Returns the time of the next sample after the last one.
    pub fn sample(&self, start: f64, dt: f64, freq: f64, out: &mut [f32]) -> f64 {
        out.fill(0.0);
        self.render_add(start, dt, freq, 1.0, out);

        return start + dt * (out.len() as f64);
    }

    // Add weighted samples to an existing buffer.
    // This is the key operation for efficient composition.
    fn render_add(&self, start: f64, dt: f64, freq: f64, weight: f32, out: &mut [f32]) {

        match self {
            Self::SineSqr { ratio } => {
                let frequency = freq * ratio;
                let mut phase = (TAU * frequency * start).rem_euclid(TAU);
                let step = TAU * frequency * dt;

                for sample in out.iter_mut() {
                    let v = phase.sin() as f32;
                    *sample += weight * v * v;
                    phase += step;
                    if phase >= TAU { phase -= TAU; }
                }
            }
            Self::Sine { ratio } => {
                let frequency = freq * ratio;
                let mut phase = (TAU * frequency * start).rem_euclid(TAU);
                let step = TAU * frequency * dt;

                for sample in out.iter_mut() {
                    *sample += weight * phase.sin() as f32;
                    phase += step;
                    if phase >= TAU { phase -= TAU; }
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

            Self::Mix(components) => {
                for (w, sound) in components {
                    sound.render_add(start, dt, freq, weight * w, out);
                }
            }
        }
    }
}