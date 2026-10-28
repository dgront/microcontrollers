use crate::notes::Note;

/// A few frequently used scales are defined in this module.
///
/// Each scale is represented as a sequence of notes, and the frequencies of these notes
/// can be calculated for any given octave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    CMajor,
    CMinorHarmonic,
    AMajor,
    AMinorHarmonic,
    GMajor,
    GMinorHarmonic,
    FMajor,
    FMinorHarmonic,
}

impl Scale {
    /// Returns the notes of the scale as an array of tuples.
    ///
    /// Each tuple contains a `Note` and its octave offset.
    pub const fn notes(self) -> &'static [(Note, u8); 8] {
        use Note::*;

        match self {
            Self::CMajor => &[
                (C, 0), (D, 0), (E, 0), (F, 0),
                (G, 0), (A, 0), (B, 0), (C, 1),
            ],
            Self::CMinorHarmonic => &[
                (C, 0), (D, 0), (Ds, 0), (F, 0),
                (G, 0), (Gs, 0), (B, 0), (C, 1),
            ],
            Self::AMajor => &[
                (A, 0), (B, 0), (Cs, 1), (D, 1),
                (E, 1), (Fs, 1), (Gs, 1), (A, 1),
            ],
            Self::AMinorHarmonic => &[
                (A, 0), (B, 0), (C, 1), (D, 1),
                (E, 1), (F, 1), (Gs, 1), (A, 1),
            ],
            Self::GMajor => &[
                (G, 0), (A, 0), (B, 0), (C, 1),
                (D, 1), (E, 1), (Fs, 1), (G, 1),
            ],
            Self::GMinorHarmonic => &[
                (G, 0), (A, 0), (As, 0), (C, 1),
                (D, 1), (Ds, 1), (Fs, 1), (G, 1),
            ],
            Self::FMajor => &[
                (F, 0), (G, 0), (A, 0), (As, 0),
                (C, 1), (D, 1), (E, 1), (F, 1),
            ],
            Self::FMinorHarmonic => &[
                (F, 0), (G, 0), (Gs, 0), (As, 0),
                (C, 1), (Cs, 1), (E, 1), (F, 1),
            ],
        }
    }

    /// Returns the frequencies of the notes in the scale for a given octave.
    pub fn frequencies(self, octave: u16) -> Vec<f64> {
        self.notes()
            .iter()
            .map(|&(note, offset)| {
                note.octave_frequency(octave + offset as u16)
            })
            .collect()
    }
}