use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Note {
    C  = 0,
    Cs = 1,
    D  = 2,
    Ds = 3,
    E  = 4,
    F  = 5,
    Fs = 6,
    G  = 7,
    Gs = 8,
    A  = 9,
    As = 10,
    B  = 11,
}

impl Note {
    /// Frequency in Hz, assuming A4 = 440 Hz.
    pub const fn frequency(self) -> f64 {
        match self {
            Self::C  => 261.63,
            Self::Cs => 277.18,
            Self::D  => 293.66,
            Self::Ds => 311.13,
            Self::E  => 329.63,
            Self::F  => 349.23,
            Self::Fs => 369.99,
            Self::G  => 392.00,
            Self::Gs => 415.30,
            Self::A  => 440.00,
            Self::As => 466.16,
            Self::B  => 493.88,
        }
    }

    pub fn octave_frequency(self, octave: u16) -> f64 {
        self.frequency() * 2.0_f64.powi(octave as i32 - 4)
    }
}


impl FromStr for Note {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C"         => Ok(Note::C),
            "C#" | "Db" => Ok(Note::Cs),
            "D"         => Ok(Note::D),
            "D#" | "Eb" => Ok(Note::Ds),
            "E"         => Ok(Note::E),
            "F"         => Ok(Note::F),
            "F#" | "Gb" => Ok(Note::Fs),
            "G"         => Ok(Note::G),
            "G#" | "Ab" => Ok(Note::Gs),
            "A"         => Ok(Note::A),
            "A#" | "Bb" => Ok(Note::As),
            "B"         => Ok(Note::B),
            _ => Err(format!("Unknown note: {s}")),
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Pitch {
    pub note: Note,
    pub octave: u16,
}

impl Pitch {
    pub fn frequency(self) -> f64 {
        self.note.octave_frequency(self.octave)
    }
}

