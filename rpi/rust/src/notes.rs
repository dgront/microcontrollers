pub struct Notes {
    octave: u16,
}

impl Notes {
    pub const C_MAJOR: &'static str = "C D E F G A B + C";
    pub const C_MINOR: &'static str = "C D Eb F G Ab Bb + C";

    pub const A_MAJOR: &'static str = "A B + C# D E F# G# A";
    pub const A_MINOR: &'static str = "A B + C D E F G# A";

    pub const G_MAJOR: &'static str = "G A B + C D E F# G";
    pub const G_MINOR: &'static str = "G A Bb + C D Eb F G";
    pub const F_MAJOR: &'static str = "F G A Bb + C D E F";
    pub const F_MINOR: &'static str = "F G Ab Bb + C Db Eb F";

    pub fn new(octave: u16) -> Self {
        Notes { octave }
    }

    /// Frequency in Hz, assuming A4 = 440 Hz.
    pub fn frequency(&self, note: &str) -> f32 {
        let semitone = match note {
            "C"  => 0,
            "C#" | "Db" => 1,
            "D"  => 2,
            "D#" | "Eb" => 3,
            "E"  => 4,
            "F"  => 5,
            "F#" | "Gb" => 6,
            "G"  => 7,
            "G#" | "Ab" => 8,
            "A"  => 9,
            "A#" | "Bb" => 10,
            "B"  => 11,
            _ => panic!("Unknown note: {note}"),
        };

        let midi = 12 * (self.octave as i32 + 1) + semitone;
        440.0 * 2.0_f32.powf((midi - 69) as f32 / 12.0)
    }

    /// Accepts "C E G", "C# D# F#" or "CEG".
    pub fn frequencies(&mut self, notes: &str) -> Vec<f32> {
        let mut result = Vec::new();
        let mut chars = notes.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '+' => self.octave += 1,
                '-' => self.octave -= 1,
                c if c.is_whitespace() => continue,
                _ => {
                    let mut note = c.to_string();

                    if matches!(chars.peek(), Some('#' | 'b')) {
                        note.push(chars.next().unwrap());
                    }

                    result.push(Self::frequency(&note, self.octave));
                }
            }
        }

        result
    }
}