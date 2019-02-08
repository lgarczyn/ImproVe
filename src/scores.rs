use crate::dissonance;
use crate::frequency::Frequency;

use crate::notes::{Note, NOTE_COUNT};

pub struct Scores {
    pub notes: [f32; NOTE_COUNT],
    pub fourier: Vec<Frequency>,
}

pub struct ScoreCalculator {
    dissonance_values: Vec<Vec<f32>>
}

impl ScoreCalculator {
    pub fn new(heard:&[Frequency]) -> ScoreCalculator {

        let dissonance_values = dissonance::dissonance_scores(heard);
        
        ScoreCalculator {
            dissonance_values
        } 
    }

    pub fn calculate_note(&self, heard:&[Frequency], note:Note) -> f32 {
        let mut score = 0f32;
        for (i, &f) in heard.iter().enumerate() {

            //TODO remove magic number
            // ignore low-intensity frequencies
            if f.intensity < 0.0001f32 {
                continue;
            }
            score += f.intensity * self.dissonance_values[note as usize][i];
        }
        score
    }

    pub fn calculate(&self, heard:Vec<Frequency>) -> Scores {
        let mut notes = [0f32; NOTE_COUNT];

        for note in Note::iter() {
            let score = self.calculate_note(heard.as_slice(), note);
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
        let amplitude = max - min + 0.0001f32;
        // Normalize score
        for score in notes.iter_mut() {
            *score = (*score - min) / amplitude;

        
            if score.is_nan() {
                println!("WTF {} {} {} {}", score, min, max, amplitude);
                panic!();
            }
        }

        Scores{notes, fourier:heard}
    }
}
