use rustfft::num_complex::Complex;
use crate::dissonance;

use crate::notes::{NOTE_COUNT, Note};

pub struct Scores {
    pub notes:[f32; NOTE_COUNT],
    pub fourier: Vec<Complex<f32>>,
}

pub fn calculate(frequencies: Vec<Complex<f32>>) -> Scores {
    let mut notes = [0f32; NOTE_COUNT];

    // For every note, calculate dissonance score
    for note in Note::iter() {
        let mut score = 0f32;
        let hz = note.freq();
        for &Complex { re: a, im: b } in frequencies.iter() {
            score += dissonance::estimate(a, b, hz);
        }
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
    // Set a min value of 5000 to avoid amplifying noise
    let amplitude = max - min;//).max(5000f32);
    // Normalize score
    for score in notes.iter_mut() {
        *score = (*score - min) / amplitude;
        // FACTOR H square rooting scores 
        //*score = score.powf(0.5f32);
    }
    Scores{notes, fourier:frequencies}
}
