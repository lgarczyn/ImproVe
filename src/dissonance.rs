/*

Dissonance.rs helps calculating the dissonance between an array of frequency-intensity couples
and a virtual idealized note played on a harmonic-rich instrument.

Currently it approximates the dissonance between a single frequency and an instrument.
The lookup table was extracted from a graph, and is therefore less than accurate.

Further improvements include a more scientific data source, varying instruments.

*/

use crate::component::Component;
use crate::notes::{Note, NOTE_COUNT};
use itertools::Itertools;
use std::f32::consts;

// Plomp-Levelt dissonance formula
// Calculates the perceived dissonance between two pure frequencies
// source: https://books.google.fr/books?id=W2_n1R5F2XoC&lpg=PA202&ots=Pp8UydRXiK&dq=%22plomb-levelt%22%20curve%20formula&pg=PA202#v=onepage&q&f=false

// The consts from the PL formula
const A: f32 = 3.5;
const B: f32 = 5.75;
const D_S: f32 = 0.24;
const S1: f32 = 0.021;
const S2: f32 = 19.0;

// The formular
pub fn dissonance(f_1: f32, f_2: f32) -> f32 {
    // The consts from the PL formula

    const A: f32 = 3.5;
    const B: f32 = 5.75;
    const D_S: f32 = 0.24;
    const S1: f32 = 0.021;
    const S2: f32 = 19.0;

    let s = D_S / (S1 * f_1.min(f_2) + S2);

    let exp = (f_2 - f_1).abs() * s;

    // Find out why curve maximum doesn't reach 1.0 without fix
    consts::E.powf(-A * exp) - consts::E.powf(-B * exp)
}

pub fn dissonance_opt(f_1: f32, s_1: f32, f_2: f32, s_2: f32) -> f32 {
    // Chose the cached 's' value of the smallest frequency
    let s = if f_1 > f_2 { s_1 } else { s_2 };

    //assert!(s == (D_S / (S1 * f_1.min(f_2) + S2)));

    // Calculate the 'exp' component
    let exp = (f_1 - f_2).abs() * s;
    // If result is negligeable, return 0
    if exp > 1f32 {
        0f32
    } else {
        consts::E.powf(-A * exp) - consts::E.powf(-B * exp)
    }
}

// Returns a 2D array mapping played notes and frequency index to dissonance score
pub fn dissonance_scores(heard: &[Component]) -> Vec<Vec<f32>> {
    // Note that the intensity of the 'heard' frequency is ignored here
    // We are only building a table of the scores of those frequencies

    // Get instrument frequencies
    let harmonics = get_notes_harmonics();

    // For each heard frequency, cache the 's' value of the PL curve
    let heard_buffered = heard
        .iter()
        .cloned()
        .map(|f| {
            let s = D_S / (S1 * f.frequency + S2);
            (f, s)
        })
        .collect_vec();

    // Prepare the 2D array
    let mut scores = vec![Vec::with_capacity(heard.len()); NOTE_COUNT];

    // For every note the user could play
    for note in Note::iter() {
        let played = harmonics[note as usize];

        // For every played frequencies, cache the same s value
        let played_buffered = played
            .iter()
            .cloned()
            .map(|f| {
                let s = D_S / (S1 * f.frequency + S2);
                (f, s)
            })
            .collect_vec();

        // For every frequency heard, calculate the dissonance to the note
        for (f_h, s_h) in heard_buffered.iter().cloned() {
            // Accumulator for the dissonance to the note
            let mut heard_score = 0f32;

            // For every frequency of the note
            for (f_p, s_p) in played_buffered.iter().cloned() {
                //Calculate dissonance
                let res = dissonance_opt(f_h.frequency, s_h, f_p.frequency, s_p);

                // Add the dissonance scores to the heard frequency score
                heard_score += res * f_p.intensity;
            }
            // Push to the lookup table
            scores[note as usize].push(heard_score);
        }
    }
    scores
}

// The number of harmonics to generate on one side of the main frequency
const HARMONIC_COUNT: usize = 300;
const FC: usize = HARMONIC_COUNT * 2 + 1;

// Get a simulated instrument's frequency components
pub fn get_notes_harmonics() -> [[Component; FC]; NOTE_COUNT] {
    let mut array: [[Component; FC]; NOTE_COUNT] = [[Component::default(); FC]; NOTE_COUNT];

    for note in Note::iter() {
        let f = note.freq();
        for i in 0..FC {
            let intensity;
            let frequency;
            let factor;

            if i == HARMONIC_COUNT {
                frequency = f;
                intensity = 1f32;
            } else if i < HARMONIC_COUNT {
                factor = (HARMONIC_COUNT + 1 - i) as f32;
                frequency = f / factor;
                intensity = 1f32 / factor; //.powf(0.5f32);
            } else {
                factor = (i + 1 - HARMONIC_COUNT) as f32;
                frequency = f * factor;
                intensity = 1f32 / factor; //.powf(0.5f32);
            }
            array[note as usize][i] = Component {
                frequency,
                intensity,
            };
        }
    }
    array
}
