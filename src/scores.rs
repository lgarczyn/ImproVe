use crate::dissonance;
use rustfft::num_complex::Complex;

use crate::notes::{Note, NOTE_COUNT};

pub struct Scores {
	pub notes: [f32; NOTE_COUNT],
	pub fourier: Vec<Complex<f32>>,
}

pub fn calculate(frequencies: Vec<Complex<f32>>) -> Scores {
	let mut notes = [0f32; NOTE_COUNT];

	// For every note, calculate dissonance score
	for note in Note::iter() {
		let mut score = 0f32;
		let hz = note.freq();
		for &Complex { re: a, im: b } in frequencies.iter() {
			score += dissonance::estimate(a, hz) * b;
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
	let amplitude = max - min;
	// Normalize score
	for score in notes.iter_mut() {
		*score = (*score - min) / amplitude;
	}
	Scores {
		notes,
		fourier: frequencies,
	}
}
