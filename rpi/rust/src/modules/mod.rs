mod module;
pub use module::*;

mod lfo;
pub use lfo::*;

mod oscillator;
pub use oscillator::*;

mod envelope;
pub use envelope::*;

mod filters;
pub use filters::*;

mod unison_oscillator;
pub use unison_oscillator::*;

mod chorus;
pub use chorus::*;

/// Calculates the frequency of a note given a base frequency, an octave ratio, and a detune fraction in semitones.
pub fn calculate_frequency(f_base: f64, octave_ratio: f64, detune_fraction: f64) -> f64 {

    // Detuning is specified in fractions of a semitone, where 12 semitones make an octave.
    let detune_factor = 2.0f64.powf(detune_fraction / 12.0);

    f_base * octave_ratio * detune_factor
}


