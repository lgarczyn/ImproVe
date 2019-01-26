use rustfft::num_complex::Complex;
use crate::dissonance;

pub const NOTE_COUNT: usize = 89;
pub const BASE_NOTE: usize = 12 * 4 + 10; // C1 + 4 octaves + 10 == A4
pub const BASE_FREQUENCY: f32 = 440f32; // Frequency of A4

pub fn calculate(frequencies: Vec<Complex<f32>>) -> [f32; NOTE_COUNT] {
    let mut scores = [0f32; NOTE_COUNT];
    let mut min = std::f32::INFINITY;
    let mut max = std::f32::NEG_INFINITY;

    // For every note, calculate dissonance score
    for i in 0 .. NOTE_COUNT {
        let mut score = 0f32;
        let diff_a: i32 = i as i32 - BASE_NOTE as i32;
        let hz = BASE_FREQUENCY * 2f32.powf(diff_a as f32 / 12f32);
        // For Complex{re:a, im:b} in [220, 440, 880].iter().map(|&hz| Complex{re:hz as f32, im:100f32})
        for &Complex { re: a, im: b } in frequencies.iter() {
            score += dissonance::estimate(a, hz) * b;
        }
        min = min.min(score);
        max = max.max(score);
        scores[i] = score;
    }
    // Get amplitude to normalize
    // Set a min value of 5000 to avoid amplifying noise
    let amplitude = max - min;//).max(5000f32);
    // Normalize score
    for score in scores.iter_mut() {
        *score = (*score - min) / amplitude;
        *score = score.powf(0.5f32);
    }
    scores
}
