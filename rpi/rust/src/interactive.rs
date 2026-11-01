
use std::io::{self, BufRead, Write};
use std::sync::mpsc::{self, Sender};

#[cfg(target_os = "linux")]
use alsa::pcm::{Access, Format, HwParams, PCM};
#[cfg(target_os = "linux")]
use alsa::{Direction, ValueOr};

#[cfg(not(target_os = "linux"))]
use crate::devices::MockupPCM as PCM;
use crate::filters::Filter;
use crate::Note;
use crate::sound_waves::{Sound, Instrument};
use crate::settings;

enum Command {
    Play(Vec<Note>, u16),
    Release,
    Quit,
}

fn interactive(tx: Sender<Command>) -> io::Result<()> {
    let mut octave = 4_u16;
    let stdin = io::stdin();

    loop {
        print!("[octave {octave}] > ");
        io::stdout().flush()?;

        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            break;
        }

        match line.trim() {
            "+" => octave = octave.saturating_add(1),
            "-" => octave = octave.saturating_sub(1),
            "" => {
                if tx.send(Command::Release).is_err() { break; }
            }
            "q" | "quit" => break,
            input => {
                match parse_chord(input) {
                    Ok(notes) => {
                        if tx.send(Command::Play(notes, octave)).is_err() { break; }
                    }
                    Err(err) => eprintln!("{err}"),
                }
            }
        }
    }

    let _ = tx.send(Command::Quit);
    Ok(())
}

fn parse_chord(input: &str) -> Result<Vec<Note>, String> {
    let mut chars = input.chars().peekable();
    let mut notes = Vec::new();

    while let Some(c) = chars.next() {
        if !matches!(c, 'A'..='G') {
            return Err(format!("Invalid note: {c}"));
        }

        let mut name = c.to_string();

        if matches!(chars.peek(), Some('#' | 'b')) {
            name.push(chars.next().unwrap());
        }

        notes.push(
            name.parse::<Note>()
                .map_err(|_| format!("Invalid note: {name}"))?
        );
    }

    Ok(notes)
}


pub fn play_interactive(pcm: &PCM, instrument: Instrument, filters: Vec<Filter>)
            -> Result<(), Box<dyn std::error::Error>> {

    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        if let Err(err) = interactive(tx) { eprintln!("Input error: {err}"); }
    });

    let io = pcm.io_i16()?;
    let mut sounds: Vec<Sound> = Vec::new();

    let mut mono = [0_i16; settings::BLOCK];
    let mut scratch = [0_i16; settings::BLOCK];
    let mut signals = [0_i16; settings::BLOCK * settings::CHANNELS];

    loop {
        // Process all pending keyboard commands
        while let Ok(command) = rx.try_recv() {
            match command {
                Command::Play(notes, octave) => {
                    for note in notes {
                        let mut sound = Sound::new(note, octave, instrument.clone());
                        sound.filters.extend(filters.iter().cloned());
                        sounds.push(sound);
                    }
                }
                Command::Release => {
                    for sound in &mut sounds {
                        sound.release();
                    }
                }
                Command::Quit => return Ok(()),
            }
        }

        sounds.retain(|sound| !sound.is_finished());

        // Generate and mix active sounds
        mono.fill(0);

        for sound in &mut sounds {
            scratch.fill(0);
            sound.sample(10_000.0, &mut scratch);

            for (dst, src) in mono.iter_mut().zip(&scratch) {
                *dst = dst.saturating_add(*src);
            }
        }

        // Mono -> interleaved stereo
        for (src, frame) in mono.iter().zip(signals.chunks_exact_mut(settings::CHANNELS)) {
            frame.fill(*src);
        }

        // Send audio to ALSA
        let mut written = 0;
        while written < settings::BLOCK {
            let n = io.writei(&signals[written * settings::CHANNELS..])?;

            if n == 0 { return Err("ALSA writei() returned 0 frames".into()); }

            written += n;
        }
    }
}
