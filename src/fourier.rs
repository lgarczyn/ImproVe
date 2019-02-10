// Standard
use std::sync::mpsc::Sender;
use std::vec;

// Tools
use itertools::Itertools;

// Math
use rustfft::num_complex::Complex;
use rustfft::FFTplanner;

//Crate
use crate::audio_buffer::AudioBuffer;
use crate::frequency::Frequency;
use crate::scores::{ScoreCalculator, Scores};


#[derive(Clone, Copy, Debug, Default)]
pub struct ScoringOptions {
    // The frequency of the audio input
    pub frequency: i32,
    // If the data should be padded for more precision
    pub zpadding: u32,
    // The time for the perceived dissonance to drop by half
    pub halflife: f32,
}

// Receives audio input, start FFT on most recent data and send results
pub fn fourier_thread(buffer: AudioBuffer, sender: Sender<Scores>, options:ScoringOptions) {
    // The FFT pool, allows for optimized yet flexible data sizes
    let mut planner = FFTplanner::<f32>::new(false);
    // The audio buffer, to get uniformly-sized audio packets
    let mut buffer = buffer;

    println!("Gathering noise profile and buffering instrument");
    // Get the first first few seconds of recording
    let vec = buffer.take().unwrap();
    // Extract frequencies to serve as mask
    let fourier = fourier_analysis(&vec[..], &mut planner, None, options);
    let mask = Some(fourier.as_slice());
    // Create a dissonance calculator from the frequencies
    let mut calculator = ScoreCalculator::new(fourier.as_slice());

    // Start analysis loop
    println!("Starting analysis");
    // While audio buffer can still output data
    while let Some(vec) = buffer.take() {
        // Apply fft and extract frequencies
        let fourier = fourier_analysis(&vec[..], &mut planner, mask, options);
        // Calculate dissonance of each note
        let scores = calculator.calculate(fourier, options.halflife);
        // Send
        sender.send(scores).ok();
    }
}

fn fourier_analysis(
    vec: &[f32],
    planner: &mut FFTplanner<f32>,
    mask: Option<&[Frequency]>,
    options: ScoringOptions,
) -> Vec<Frequency> {

    // Setup fft parameters, possibly padding the input array
    let len = vec.len() * options.zpadding as usize;
    let mut fft_in = vec
        .iter()
        .map(|&f| Complex { re: f, im: 0f32 })
        .collect_vec();
    fft_in.resize(len, Complex::default());
    let mut fft_out = vec![Complex::default(); len];
    let fft = planner.plan_fft(len);

    // Process fft
    fft.process(&mut fft_in, &mut fft_out);

    // Discard useless data
    fft_out.truncate(len / 2);
    // Map results to frequencies and intensity, skipping the first element (0hz)
    fft_out
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, c)| {
            // Calculate intensity
            // FACTOR A norm_sqr vs sqr ?
            let mut intensity = c.norm_sqr();
            // Calculate frequency
            let frequency = i as f32 / len as f32 * options.frequency as f32;
            // Noise masking, currently unused
            if let Some(vec) = mask {
                if intensity > vec[i - 1].intensity {
                    intensity -= vec[i - 1].intensity;
                } else {
                    intensity = 0f32;
                }
            }
            // Reducing intensity of frequencies out of human hearing range
            // FACTOR B a_weighing and how much
            intensity *= a_weigh_frequency(c.re);
            // Build intensity/value couple
            Frequency {
                intensity,
                value: frequency,
            }
        })
        .collect_vec()
}

// https://fr.mathworks.com/matlabcentral/fileexchange/46819-a-weighting-filter-with-matlab
// Reduce frequency intensity based on human perception
pub fn a_weigh_frequency(freq: f32) -> f32 {
    let c1 = 12194.217f32.powi(2);
    let c2 = 20.598_997f32.powi(2);
    let c3 = 107.652_65f32.powi(2);
    let c4 = 737.862_24f32.powi(2);
    // Evaluate the A-weighting filter in the frequency domain
    let freq = freq.powi(2);
    let num = c1 * (freq.powi(2));
    let den = (freq + c2) * ((freq + c3) * (freq + c4)).sqrt() * (freq + c1);
    1.2589f32 * num / den
}
