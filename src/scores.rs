use crate::dissonance;
use crate::frequency::Frequency;

use crate::notes::{Note, NOTE_COUNT};

use itertools::Itertools;

pub struct Scores {
    pub notes: [f32; NOTE_COUNT],
    pub fourier: Vec<Frequency>,
}

pub struct ScoreCalculator {
    dissonance_values: Vec<Vec<f32>>,
    prev_score: [f32; NOTE_COUNT],
}

impl ScoreCalculator {
    pub fn new(heard:&[Frequency]) -> ScoreCalculator {

        let dissonance_values = dissonance::dissonance_scores(heard);
        
        ScoreCalculator {
            dissonance_values,
            prev_score: [0f32; NOTE_COUNT],
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
        for note in Note::iter() {
            let score = self.calculate_note(heard_sorted.as_slice(), note);
            //if score > self.prev_score[note as usize] {
            //    notes[note as usize] = score * 0.5 + self.prev_score[note as usize] * 0.5;
            //} else {
                notes[note as usize] = score * 0.1 + self.prev_score[note as usize] * 0.9;
            //}
        }

        self.prev_score = notes;

        // Octave average (loses ton of information, but avoids eye strain in low-constrast regions)

        let mut avg = [0f32; 12];

        for (i, &score) in notes.iter().enumerate() {
            avg[i % 12] += score / (notes.len() as f32 / 12.0);
        }

        for (i, score) in notes.iter_mut().enumerate() {
            *score = avg[i % 12];
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
