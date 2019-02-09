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
        // Skip lower half for noise reduction
        let heard_sorted = heard.iter()
            .cloned()
            .enumerate()
            .sorted_by_key(|(_, f)| *f)
            .skip(heard.len() / 2)
            .skip(heard.len() / 4)
            .skip(heard.len() / 8)
            .collect_vec();

        for note in Note::iter() {
            let score = self.calculate_note(heard_sorted.as_slice(), note);
            notes[note as usize] = score * 0.1 + self.prev_score[note as usize] * 0.9;
        }

        self.prev_score = notes;

        let mut average = notes[0];

        for score in notes.iter_mut() {
            average = average * 0.7f32 + *score * 0.3f32;
            *score -= average; 
        }

        let (min, max) = notes
            .iter().cloned()
            .minmax()
            .into_option()
            .unwrap();
        
        // Get amplitude to normalize
        let amplitude = max - min + 0.0001f32;
        // Normalize score
        for score in notes.iter_mut() {
            *score = (*score - min) / amplitude;

        
            if score.is_nan() {
                println!("WTF {} {} {} {}", score, min, max, amplitude);
            }
        }

        Scores{notes, fourier:heard}
    }
}
