use crate::dissonance;
use crate::frequency::Frequency;

use crate::notes::{Note, NOTE_COUNT};

use itertools::Itertools;

use std::time::Instant;

pub struct Scores {
    pub notes: [f32; NOTE_COUNT],
    pub fourier: Vec<Frequency>,
}

pub struct ScoreCalculator {
    dissonance_values: Vec<Vec<f32>>,
    prev_score: [f32; NOTE_COUNT],
    time: Instant
}

impl ScoreCalculator {
    pub fn new(heard:&[Frequency]) -> ScoreCalculator {

        let dissonance_values = dissonance::dissonance_scores(heard);
        
        ScoreCalculator {
            dissonance_values,
            prev_score: [0f32; NOTE_COUNT],
            time: Instant::now()
        } 
    }

    pub fn calculate_note(&self, heard:&[(usize, Frequency)], note:Note) -> f32 {
        let mut score = 0f32;
        for &(u, f) in heard.iter() {
            score += f.intensity * self.dissonance_values[note as usize][u];
        }
        score
    }

    pub fn calculate(&mut self, heard:Vec<Frequency>) -> Scores {
        let mut notes = [0f32; NOTE_COUNT];

        // Extrac indices for lookup table
        // Sort the array
        // Possibly skip lower parts for noise reduction
        let heard_sorted = heard.iter()
            .cloned()
            .enumerate()
            .sorted_by_key(|(_, f)| *f)
            // .skip(heard.len() / 2)
            // .skip(heard.len() / 4)
            // .skip(heard.len() / 8)
            .collect_vec();

        // Time-wise walking average
        // An approximation of second-order beatings
        // Doesn't take into account the different type of dissonance

        // Get time since last call
        let time_since_last_call = self.time.elapsed();
        let seconds = time_since_last_call.as_secs() as f32 + time_since_last_call.subsec_nanos() as f32 * 1e-9;
        self.time = Instant::now();
        // Get how much previous score should have faded (-30% per second)
        let factor = 0.7f32.powf(seconds);
        // Apply to each score
        for note in Note::iter() {
            let score = self.calculate_note(heard_sorted.as_slice(), note);
            notes[note as usize] = score * (1f32 - factor) + self.prev_score[note as usize] * factor;
        }

        self.prev_score = notes;

        // Octave average (loses ton of information, but avoids eye strain in low-constrast regions)

        // let mut avg = [0f32; 12];

        // for (i, &score) in notes.iter().enumerate() {
        //     avg[i % 12] += score / (notes.len() as f32 / 12.0);
        // }

        // for (i, score) in notes.iter_mut().enumerate() {
        //     *score = avg[i % 12];
        // }

        // Octave rescaling
        // Moves every scale to the same range (0 .. 1)
        // Creates inconsistencies between octaves, but OK compromise

        let mut minmax = [(0f32, 0f32); NOTE_COUNT / 12 + 1];

        for (i, it) in notes.iter().chunks(12).into_iter().enumerate() {
            minmax[i] = it.cloned().minmax().into_option().unwrap();
        }

        for (i, score) in notes.iter_mut().enumerate() {
            let (min, max) = minmax[i / 12];
            *score = (*score - min) / (max - min);
        }

        // Walking average (doesn't deal with varying amplitude)
        // let mut average = notes[0];
        // for score in notes.iter_mut() {
        //     average = average * 0.7f32 + *score * 0.3f32;
        //     *score -= average; 
        // }

        Scores{notes, fourier:heard}
    }
}
