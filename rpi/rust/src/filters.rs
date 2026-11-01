use crate::settings::{RATE};

#[derive(Debug, Clone)]
pub enum Filter {
    LowPass(LowPass),
    Biquad(Biquad),
    Echo(Echo)
}

impl Filter {

    pub fn lowpass(cutoff: f64) -> Self {
        Filter::LowPass(LowPass::new(cutoff))
    }

    pub fn biquad_lowpass(cutoff: f64, q: f64) -> Self {
        Filter::Biquad(Biquad::new(cutoff, q))
    }

    pub fn echo(delay_seconds: f64, feedback: f64, mix: f64) -> Self {
        Filter::Echo(Echo::new(delay_seconds, feedback, mix))
    }

    pub fn apply(&mut self, samples: &mut [f32]) {
        match self {
            Filter::LowPass(f) => f.apply(samples),
            Filter::Biquad(f) => f.apply(samples),
            Filter::Echo(f) => f.apply(samples)
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

