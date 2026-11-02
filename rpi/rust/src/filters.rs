use crate::settings::{RATE};

#[derive(Debug, Clone)]
pub enum Filter {
    LowPass(LowPass),
    Biquad(Biquad),
    MoogLadder(MoogLadder),
    Echo(Echo)
}

impl Filter {

    pub fn lowpass(cutoff: f64) -> Self {
        Filter::LowPass(LowPass::new(cutoff))
    }

    pub fn biquad_lowpass(cutoff: f64, q: f64) -> Self {
        Filter::Biquad(Biquad::new(cutoff, q))
    }

    pub fn moog_ladder(cutoff: f64, q: f64) -> Self {
        Filter::MoogLadder(MoogLadder::new(cutoff, q))
    }

    pub fn echo(delay_seconds: f64, feedback: f64, mix: f64) -> Self {
        Filter::Echo(Echo::new(delay_seconds, feedback, mix))
    }

    pub fn apply(&mut self, samples: &mut [f32]) {
        match self {
            Filter::LowPass(f) => f.apply(samples),
            Filter::Biquad(f) => f.apply(samples),
            Filter::Echo(f) => f.apply(samples),
            Filter::MoogLadder(f) => f.apply(samples)
        }
    }
}

#[derive(Debug, Clone)]
pub struct LowPass {
    alpha: f32,
    previous: f32,
}

impl LowPass {
    pub fn new(cutoff: f64) -> Self {
        let alpha = (1.0 - (-2.0 * std::f64::consts::PI * cutoff / RATE as f64).exp()) as f32;

        Self { alpha, previous: 0.0, }
    }

    pub fn apply(&mut self, samples: &mut [f32]) {
        for x in samples {
            self.previous += self.alpha * (*x - self.previous);
            *x = self.previous;
        }
    }
}


#[derive(Debug, Clone)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    pub fn new(cutoff: f64, q: f64) -> Self {
        let w = 2.0 * std::f64::consts::PI * cutoff / RATE as f64;
        let (sin, cos) = w.sin_cos();
        let alpha = sin / (2.0 * q);
        let a0 = 1.0 + alpha;

        Self {
            b0: ((1.0 - cos) / (2.0 * a0)) as f32,
            b1: ((1.0 - cos) / a0) as f32,
            b2: ((1.0 - cos) / (2.0 * a0)) as f32,
            a1: (-2.0 * cos / a0) as f32,
            a2: ((1.0 - alpha) / a0 )as f32,
            z1: 0.0,
            z2: 0.0,
        }
    }

    pub fn apply(&mut self, samples: &mut [f32]) {

        for x in samples {
            let y = self.b0 * *x + self.z1;
            self.z1 = self.b1 * *x - self.a1 * y + self.z2;
            self.z2 = self.b2 * *x - self.a2 * y;
            *x = y;
        }
    }
}

#[derive(Debug, Clone)]
pub struct MoogLadder {
    pub cutoff: f64,
    pub resonance: f64,
    // Memory for the 4 stages of the ladder filter
    s: [f64; 4],
}

impl MoogLadder {
    pub fn new(cutoff: f64, resonance: f64) -> Self {
        // Resonance is clamped to avoid instability in the filter
        Self { cutoff, resonance: resonance.clamp(0.0, 3.95), s: [0.0; 4] }
    }

    pub fn apply(&mut self, audio_buffer: &mut [f32]) {

        let g = (std::f64::consts::PI * self.cutoff / RATE as f64).tan();
        let h = g / (1.0 + g);
        for sample in audio_buffer.iter_mut() {

            let input = *sample as f64;

            /// Decay
            let s0 = self.s[0] / (1.0 + g);
            let s1 = self.s[1] / (1.0 + g);
            let s2 = self.s[2] / (1.0 + g);
            let s3 = self.s[3] / (1.0 + g);

            let s_sum = h * h * h * s0 + h * h * s1 + h * s2 + s3;

            // 3. Główne równanie pętli sprzężenia zwrotnego (Zero-Delay Feedback)
            let mut g_pow4 = h * h;
            g_pow4 *= g_pow4; // h^4
            let denominator = 1.0 + self.resonance * g_pow4;
            let u = (input - self.resonance * s_sum) / denominator;

            // 4. Przepuszczenie sygnału przez 4 stopnie drabinowe (One-Pole TPT)
            // Stopień 1
            let v0 = (u - self.s[0]) * h;
            let out0 = v0 + self.s[0];
            self.s[0] = out0 + v0;

            // Stopień 2
            let v1 = (out0 - self.s[1]) * h;
            let out1 = v1 + self.s[1];
            self.s[1] = out1 + v1;

            // Stopień 3
            let v2 = (out1 - self.s[2]) * h;
            let out2 = v2 + self.s[2];
            self.s[2] = out2 + v2;

            // Stopień 4 (Wyjście końcowe filtra)
            let v3 = (out2 - self.s[3]) * h;
            let out3 = v3 + self.s[3];
            self.s[3] = out3 + v3;

            *sample = out3.clamp(-1.0, 1.0) as f32;
        }
    }
}

#[derive(Debug, Clone)]
pub struct Echo {
    buffer: Vec<f32>,
    position: usize,
    feedback: f32,
    mix: f32,
}

impl Echo {
    pub fn new(delay_seconds: f64, feedback: f64, mix: f64) -> Self {
        let delay_samples = (delay_seconds * RATE as f64).round() as usize;

        Self {
            buffer: vec![0.0; delay_samples.max(1)],
            position: 0,
            feedback: feedback.clamp(0.0, 0.99) as f32,
            mix: mix.clamp(0.0, 1.0) as f32,
        }
    }

    pub fn apply(&mut self, samples: &mut [f32]) {
        for sample in samples {
            let input = *sample;
            let delayed = self.buffer[self.position];

            self.buffer[self.position] = input + delayed * self.feedback;

            *sample = input + self.mix * delayed;

            self.position = (self.position + 1) % self.buffer.len();
        }
    }
}

