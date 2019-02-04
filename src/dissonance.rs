
/*

Dissonance.rs helps calculating the dissonance between an array of frequency-intensity couples
and a virtual idealized note played on a harmonic-rich instrument.

Currently it approximates the dissonance between a single frequency and an instrument.
The lookup table was extracted from a graph, and is therefore less than accurate.

Further improvements include a more scientific data source, varying instruments.

*/


use std::f32::consts;
use itertools::Itertools;
use crate::frequency::Frequency;
use crate::notes::{Note, NOTE_COUNT};

// Plomp-Levelt dissonance formula
// Calculates the perceived dissonance between two pure frequencies
// source: https://books.google.fr/books?id=W2_n1R5F2XoC&lpg=PA202&ots=Pp8UydRXiK&dq=%22plomb-levelt%22%20curve%20formula&pg=PA202#v=onepage&q&f=false

pub fn dissonance(f_1:f32, f_2:f32, ) -> f32 {

	// The consts from the PL formula

	const A:f32 = 3.5;
	const B:f32 = 5.75;
	const D_S:f32 = 0.24;
	const S1:f32 = 0.021;
	const S2:f32 = 19.0;

	let s = D_S / (S1 * f_1.min(f_2) + S2);

	let exp = (f_2 - f_1).abs() * s;

	// Find out why curve maximum doesn't reach 1.0 without fix
	consts::E.powf(-A * exp) - consts::E.powf(-B * exp)
}

// Calculates the dissonance between a list of frequencies and a synthesized note
pub fn dissonance_note(heard:&[Frequency], note:Note) -> f32 {

	dissonance_complex(heard, &NOTE_HARMONICS[note as usize])
}

// The same function as dissonance, but optimized for large amount of data
pub fn dissonance_complex(heard:&[Frequency], played:&[Frequency]) -> f32 {
	const D_S:f32 = 0.24;
	const S1:f32 = 0.021;
	const S2:f32 = 19.0;

	// For each heard frequency, cache the 's' value of the PL curve
	let heard_buffered = heard.iter().cloned().map(|f| {
		let s = D_S / (S1 * f.value + S2);
		(f, s)
	});

	// For every played frequencies, cache the same value
	let played_buffered = played.iter().cloned().map(|f| {
		let s = D_S / (S1 * f.value + S2);
		(f, s)
	}).collect_vec();

	// Iterate on every frequency heard
	// Could be optimized if we have more information on data sortedness
	let mut score = 0f32;
	for (f_h, s_h) in heard_buffered {
		// Accumulates the score for a single heard frequency
		let mut heard_score = 0f32;

		// TODO: remove magic numbers
		// ignore barely audible frequencies
		if f_h.intensity < 0.0001f32 {
			continue;
		}

		// Iterate again on every predicted frequencies
		for (f_p, s_p) in played_buffered.iter().cloned() {
			// Chose the cached 's' value of the smallest frequency
			let s = if f_h.value > f_p.value {
				s_p
			} else {
				s_h
			};

			//assert!(s == (D_S / (S1 * f_h.min(f_p) + S2)));
			
			// Calculate the 'exp' component
			let exp = (f_h.value - f_p.value).abs() * s;
			if exp > POW_MAX_INPUT as f32 {
				continue;
			}
			// Use component as index to lookup table
			let res = POW_LOOKUP[(exp * POW_RESOLUTION as f32) as usize];

			//assert!((res - consts::E.powf(-A * exp) + consts::E.powf(-B * exp)).abs() < 0.01f32);
			
			// Add the dissonance score to the pile
			heard_score += res * f_p.intensity;
		}
		// Multiply all dissonance scores for the frequency heard
		// by its intensity, than add to global dissonance score 
		score += heard_score * f_h.intensity;
	}
	score
}

// How many lookup table elements for one unit
const POW_RESOLUTION:usize = 1000;
// The max exp value used as indexing
const POW_MAX_INPUT:usize = 1;
// More PL formula const, as they cannot be placed inside the lazy_static
const A:f32 = 3.5;
const B:f32 = 5.75;

lazy_static! {
	// The lookup table for the expensive last step
	static ref POW_LOOKUP:Vec<f32> = (0 .. POW_RESOLUTION * POW_MAX_INPUT)
		.map(|i| (i as f32 + 0.5) / POW_RESOLUTION as f32 )
		.map(|f| (consts::E.powf(-A * f) - consts::E.powf(-B * f)))
		.collect_vec();
	
	// The cached instruments components for each note
	static ref NOTE_HARMONICS:[[Frequency;FC]; NOTE_COUNT] = get_notes_harmonics();
}

// The number of harmonics to generate on one side of the main frequency
const HARMONIC_COUNT:usize = 30;
const FC:usize = HARMONIC_COUNT * 2 + 1;

// Get a simulated instrument's frequency components
fn get_notes_harmonics() -> [[Frequency;FC]; NOTE_COUNT] {
	let mut array:[[Frequency;FC]; NOTE_COUNT] = [[Frequency::default();FC]; NOTE_COUNT];

	for note in Note::iter() {
		let f = note.freq();
		for i in 0 .. FC {

			let intensity;
			let frequency;
			let factor;

			if i == HARMONIC_COUNT {
				frequency = f;
				intensity = 1f32;
			} else if i < HARMONIC_COUNT {
				factor = (HARMONIC_COUNT + 1 - i) as f32;
				frequency = f / factor;
				intensity = 1f32 / factor;//.powf(0.5f32);
			} else {
				factor = (i + 1 - HARMONIC_COUNT) as f32;
				frequency = f * factor;
				intensity = 1f32 / factor;//.powf(0.5f32);
			}
			array[note as usize][i] = Frequency{value:frequency, intensity};
		}
	}
	array
}
