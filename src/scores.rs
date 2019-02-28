use crate::dissonance;
use crate::frequency::Frequency;

use crate::notes::{Note, NOTE_COUNT};

use itertools::Itertools;

use std::time::Instant;

pub struct Scores {
    // The dissonance score of each note
    pub note_scores: [f32; NOTE_COUNT],
    // The intensity of each note
    pub note_values: [f32; NOTE_COUNT],
    pub fourier: Vec<Frequency>,
}

pub struct ScoreCalculator {
    dissonance_values: Vec<Vec<f32>>,
    prev_score: [f32; NOTE_COUNT],
    prev_values: [f32; NOTE_COUNT],
    time: Instant,
}

impl ScoreCalculator {
    pub fn new(heard: &[Frequency]) -> ScoreCalculator {
        let dissonance_values = dissonance::dissonance_scores(heard);

        ScoreCalculator {
            dissonance_values,
            prev_score: [0f32; NOTE_COUNT],
            prev_values: [0f32; NOTE_COUNT],
            time: Instant::now(),
        }
    }

    fn calculate_note(&self, heard: &[(usize, Frequency)], note: Note) -> f32 {
        let mut score = 0f32;
        for &(u, f) in heard.iter() {
            score += f.intensity * self.dissonance_values[note as usize][u];
        }
        score
    }

    fn calculate_scores(&mut self, heard: &Vec<Frequency>, factor:f32) -> [f32; NOTE_COUNT] {
        let mut notes = [0f32; NOTE_COUNT];

        // Extract indices for lookup table
        // Sort the array
        // Possibly skip lower parts for noise reduction
        let heard_sorted = heard
            .iter()
            .cloned()
            .enumerate()
            .sorted_by_key(|(_, f)| *f)
            // .skip(heard.len() / 2)
            // .skip(heard.len() / 4)
            // .skip(heard.len() / 8)
            .collect_vec();

        // Get each score, and average with previous value
        for note in Note::iter() {
            let score = self.calculate_note(heard_sorted.as_slice(), note);
            notes[note as usize] =
                score * (1f32 - factor) + self.prev_score[note as usize] * factor;
        }

        self.prev_score = notes;
        
        // Normalise octaves
        for i in 0 .. notes.len() {

            let octave_pos = i % 12;
            // Get index of first and last note of octave (ie. C2 and C3)
            let prev_c = i - octave_pos;
            let next_c = prev_c + 12;

            if next_c < notes.len() {

                // Get diff between the scores
                let amp = notes[prev_c] - notes[next_c];
                // Normalize lineraly
                notes[i] = notes[i] + amp * octave_pos as f32 / 12.0;
            }
        }

        // Extract the range of each octave
        let mut minmax = [(0f32, 0f32); NOTE_COUNT / 12 + 1];

        for (i, it) in notes.iter().chunks(12).into_iter().enumerate() {
            minmax[i] = it.cloned().minmax().into_option().unwrap();
        }

        // Move each octave to the 0.0 .. 1.0 range
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
        notes
    }

    // Assign each frequency to a note, and sum their value
    // Allows the dsplay of every perceived note
    fn calculate_values(&mut self, heard: &Vec<Frequency>, factor:f32) -> [f32; NOTE_COUNT] {
        let mut note_values = [0f32; NOTE_COUNT];

        for f in heard {
            if let Some(note) = Note::from_freq(f.value) {
                let index = note as usize;
                //note_values[index] += f.intensity.sqrt();
                note_values[index] =
                f.intensity.sqrt() * (1f32 - factor) + self.prev_values[note as usize] * factor;
            }
        }
        self.prev_values = note_values;
        note_values
    }

    // Analyses a list of perceived frequencies, and returns displayable data
    pub fn calculate(&mut self, heard: Vec<Frequency>, halflife:f32) -> Scores {

        // Time-aware walking average
        // An approximation of second-order beatings
        // Basically approximates how long does the brain keep "hearing" the note
        // Doesn't take into account the different type of dissonance

        // Get time since last call
        let time_since_last_call = self.time.elapsed();
        let seconds = time_since_last_call.as_secs() as f32
            + time_since_last_call.subsec_nanos() as f32 * 1e-9;
        self.time = Instant::now();
        // Get how much previous score should have faded
        let factor = 0.5f32.powf(seconds / halflife);

        Scores {
            note_scores: self.calculate_scores(&heard, factor),
            note_values: self.calculate_values(&heard, factor / 5.0),
            fourier: heard,
        }
    }
}
