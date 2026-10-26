use std::f64::consts::TAU;

pub enum Sound {
    Sine { freq: f64 },
    Sawtooth { freq: f64 },
    Square { freq: f64 },
    Mix(Vec<(f32, Sound)>),
}

impl Sound {

    pub fn sine(freq: f64) -> Self { Self::Sine { freq } }

    pub fn sawtooth(freq: f64) -> Self { Self::Sawtooth { freq } }

    pub fn square(freq: f64) -> Self { Self::Square { freq } }

    pub fn mix(components: impl IntoIterator<Item = (f32, Sound)>) -> Self {
        Self::Mix(components.into_iter().collect())
    }

    pub fn piano(freq: f64) -> Self {
        const COEFF: [f32; 10] = [1.0, 0.40, 0.25, 0.12, 0.08, 0.05, 0.03, 0.02, 0.01, 0.0];

        let norm: f32 = COEFF.iter().map(|c| c.abs()).sum();

        Self::mix(
            COEFF.iter()
                .enumerate()
                .map(|(i, &c)| {
                    (c / norm, Self::sine(freq * (i + 1) as f64))
                })
        )
    }

    /// Returns the value of the sound at a given time (in seconds).
    pub fn value(&self, time: f64) -> f32 {
        match self {
            Self::Sine { freq } => {
                (TAU * freq * time).sin() as f32
            }

            Self::Sawtooth { freq } => {
                (2.0 * (freq * time).rem_euclid(1.0) - 1.0) as f32
            }

            Self::Square { freq } => {
                if (freq * time).rem_euclid(1.0) < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }

            Self::Mix(components) => {
                components.iter()
                    .map(|(weight, sound)| weight * sound.value(time))
                    .sum()
            }
        }
    }

    // ---------- Block rendering ----------

    /// Renders a block of sound samples into the given output buffer.
    ///
    /// Sound sampling starts at `start` seconds, with a time step of `dt` seconds between samples.
    /// The number of samples is determined by the length of the `out` buffer.
    /// Returns the time of the next sample after the last one.
    pub fn sample(&self, start: f64, dt: f64, out: &mut [f32]) -> f64 {
        out.fill(0.0);
        self.render_add(start, dt, 1.0, out);

        return start + dt * (out.len() as f64);
    }

    // Add weighted samples to an existing buffer.
    // This is the key operation for efficient composition.
    fn render_add(&self, start: f64, dt: f64, weight: f32, out: &mut [f32]) {

        match self {
            Self::Sine { freq } => {
                let mut phase = (TAU * freq * start).rem_euclid(TAU);
                let step = TAU * freq * dt;

                for sample in out.iter_mut() {
                    *sample += weight * phase.sin() as f32;
                    phase += step;
                    if phase >= TAU { phase -= TAU; }
                }
            }

            Self::Sawtooth { freq } => {
                let step = freq * dt;
                let mut phase = (freq * start).rem_euclid(1.0);

                for sample in out.iter_mut() {
                    *sample += weight * (2.0 * phase - 1.0) as f32;
                    phase = (phase + step).rem_euclid(1.0);
                }
            }

            Self::Square { freq } => {
                let step = freq * dt;
                let mut phase = (freq * start).rem_euclid(1.0);

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
                    sound.render_add(start, dt, weight * w, out);
                }
            }
        }
    }
}