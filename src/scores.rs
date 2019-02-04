use crate::dissonance;
use crate::frequency::Frequency;

use crate::notes::{Note, NOTE_COUNT};

pub struct Scores {
    pub notes: [f32; NOTE_COUNT],
    pub fourier: Vec<Frequency>,
}

pub fn calculate(frequencies: Vec<Frequency>) -> Scores {
    let mut notes = [0f32; NOTE_COUNT];

    // For every note, calculate dissonance score
    for note in Note::iter() {
        let score = dissonance::dissonance_note(frequencies.as_slice(), note);
        notes[note as usize] = score;
    }

    let mut average = notes[0];

    for score in notes.iter_mut() {
        average = average * 0.7f32 + *score * 0.3f32;
        *score -= average; 
    }

    let mut min = std::f32::INFINITY;
    let mut max = std::f32::NEG_INFINITY;

    for &score in notes.iter() {
        min = min.min(score);
        max = max.max(score);
    }
    // Get amplitude to normalize
    let amplitude = max - min;
    // Normalize score
    for score in notes.iter_mut() {
        *score = (*score - min) / amplitude;
    }
    Scores{notes, fourier:frequencies}
}
