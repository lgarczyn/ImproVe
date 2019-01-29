// Standard
use std::sync::mpsc::Sender;
use std::vec;

// Tools
use itertools::Itertools;

// Math
use rustfft::num_complex::Complex;
use rustfft::FFTplanner;

//Create
use crate::audio_buffer::AudioBuffer;
use crate::scores::{Scores, calculate};

// Receives audio input, start FFT on most recent data and send results
pub fn fourier_thread(buffer:AudioBuffer, sender:Sender<Scores>) {
    // The FFT pool, allows for optimized yet flexible data sizes
    let mut planner = FFTplanner::<f32>::new(false);
    // The audio buffer, to get uniformly-sized audio packets
    let mut buffer = buffer;

    // Get the first first few seconds of recording
    println!("Gathering noise profile");
    let vec = buffer.take();
    // Extract frequencies to serve as mask
    let mask = fourier_analysis(&vec[..], &mut planner, None);

    // Start analysis loop
    println!("Starting analysis");
    loop {
        // Aggregate all pending input
        let vec = buffer.take();
        // Apply fft and extract frequencies
        let fourier = fourier_analysis(&vec[..], &mut planner, Some(&mask));
        // Calculate dissonance of each note
        let scores = calculate(fourier);
		// Send
		sender.send(scores).ok();
    }
}

fn fourier_analysis(
    vec: &[f32],
    planner: &mut FFTplanner<f32>,
    mask: Option<&Vec<Complex<f32>>>,
) -> Vec<Complex<f32>> {
    // Setup fft parameters
    let len = vec.len();
    let mut fft_in = vec
        .iter()
        .map(|&f| Complex { re: f, im: 0f32 })
        .collect_vec();
    let mut fft_out = vec![Complex::default(); len];
    let fft = planner.plan_fft(len);

    // Process fft
    fft.process(&mut fft_in, &mut fft_out);

    // Discard useless data
    fft_out.truncate(len / 2);
    fft_out.remove(0);
    // Map results to frequencies and intensity
    for (c, i) in fft_out.iter_mut().zip(1..) {
        // Calculate intensity
        c.im = c.norm_sqr();// (*a * *a + c.im * c.im).sqrt();
        // Calculate frequency
        c.re = i as f32 * 48000f32 / len as f32;
        // Noise masking, currently unused
        if let Some(vec) = mask {
            if c.im > vec[i - 1].im {
                c.im -= vec[i - 1].im;
            } else {
                c.im = 0f32;
            }
        }
        // Reducing intensity of frequencies out of human hearing range
        c.im *= a_weigh_frequency(c.re).powi(2);
    }

    // Sort by intensity
    //fft_out.sort_by(|b, a| a.im.partial_cmp(&b.im).unwrap());

    // print 9 most intense frequencies (you'll never believe number 4)
    // for &Complex{re:a, im:b} in fft_out.iter().take(9)
    // {
    //     print!("{:^5.0}:{:^6.2} ", a, b);
    // }
    // println!("");

    fft_out
}

// https://fr.mathworks.com/matlabcentral/fileexchange/46819-a-weighting-filter-with-matlab
// Reduce frequency intensity based on human perception
fn a_weigh_frequency(freq: f32) -> f32 {
    let c1 = 12194.217f32.powi(2);
    let c2 = 20.598997f32.powi(2);
    let c3 = 107.65265f32.powi(2);
    let c4 = 737.86223f32.powi(2);
    // Evaluate the A-weighting filter in the frequency domain
    let freq = freq.powi(2);
    let num = c1 * (freq.powi(2));
    let den = (freq + c2) * ((freq + c3) * (freq + c4)).sqrt() * (freq + c1);
    1.2589f32 * num / den
}
