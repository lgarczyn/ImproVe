// Standard
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use std::vec;
use std::io;
use std::io::BufWriter;
use std::io::Write;

// Tools
use clampf::clamp;
use itertools::Itertools;

// Math
use rustfft::num_complex::Complex;
use rustfft::FFTplanner;

// Audio
use cpal::Sample;
use cpal::StreamData::Input;
use cpal::UnknownTypeInputBuffer::{F32, I16, U16};

// Parser
use clap::{Arg, App};

mod dissonance;
mod audio_buffer;

fn main() {
    // Parse args
    let matches = App::new("ImproVe")
        .version("0.1")
        .author("Louis Garczynski <louis.roc@gmail.com>")
        .about("Real-time improvisation suggestions")
        .arg(Arg::with_name("resolution")
            .short("r")
            .long("resolution")
            .value_name("UINT")
            .help("Width of audio data analyzed every step\n\
                  Higher values 'blur' the audio over time\n\
                  Higher values can have a significant performance cost\n\
                  Powers of two are significantly faster\n")
            .next_line_help(true)
            .default_value("8192")
            .validator(|s| match s.parse::<u32>() {
                Ok(_) => Ok(()),
                Err(_) => Err("Argument is not an unsigned int".to_owned())
            }))
        .arg(Arg::with_name("notation")
            .short("n")
            .long("notation")
            .value_name("LANGUAGE")
            .help("English or Romance notation\n")
            .next_line_help(true)
            .possible_values(&["e", "r"])
            .default_value("e"))
        // removes because causes cpal to crash :c
        // .arg(Arg::with_name("input")
        //     .short("i")
        //     .long("input")
        //     .value_name("DEVICE")
        //     .help("The id of the audio input you wish to use\n")
        //     .next_line_help(true)
        //     .takes_value(true))
        .get_matches();
    
    // Get notation convention
    let notation = match matches.value_of("notation").unwrap()
    {
        "e" => NOTE_NAMES_ENGLISH,
        _ => NOTE_NAMES_ROMANCE,
    };

    // Get number of packets to read in a single FFT
    let resolution = matches.value_of("resolution").unwrap().parse::<u32>().unwrap();

    // Get the desired input device, default or otherwise
    let device = match matches.value_of("input")
    {
        None => cpal::default_input_device()
            .expect("Failed to get default input device"),
        Some(s) => cpal::input_devices()
            .filter(|d| d.name() == s)
            .next()
            .expect(&format!("Could not find device named '{}'. Here's the list of available devices: {}.",
                s, cpal::input_devices().map(|d| d.name()).join(", ")))
    };
    println!("Default input device: {}", device.name());

    // Get the default sound input format
    let format = device
        .default_input_format()
        .expect("Failed to get default input format");
    println!("Default input format: {:?}", format);

    // Start the cpal input stream
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop
        .build_input_stream(&device, &format)
        .expect("Failed to build input stream");
    event_loop.play_stream(stream_id);

    // The channel to send data from audio thread to fourier thread
    let (sender, receiver) = channel::<Vec<f32>>();

    // Spawn the audio input reading thread
    std::thread::spawn(move || {
        event_loop.run(move |_, data| {
            // Otherwise send input data to fourier thread
            if let Input { buffer: input_data } = data {
                let float_buffer = match input_data {
                    U16(buffer) => buffer.iter().map(|u| u.to_f32()).collect_vec(),
                    I16(buffer) => buffer.iter().map(|i| i.to_f32()).collect_vec(),
                    F32(buffer) => buffer.to_vec(),
                };
                sender.send(float_buffer).unwrap();
            }
        });
    });

    // Spawn the audio analysis thread
    thread::spawn(move || fourier_thread(receiver, notation, resolution as usize));

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();
}

// Receives audio input, start FFT on most recent data and display results
fn fourier_thread(
    receiver: Receiver<Vec<f32>>,
    notation: [&str; 12],
    resolution: usize) {
    // The FFT pool, allows for optimized yet flexible data sizes
    let mut planner = FFTplanner::<f32>::new(false);
    // The audio buffer, to always get N elements
    let mut buffer = audio_buffer::AudioBuffer::new(receiver);

    // Get the first first few seconds of recording
    println!("Gathering noise profile");
    let vec = buffer.take(resolution);
    // Extract frequencies to serve as mask
    let mask = fourier_analysis(&vec[..], &mut planner, None);

    // Fill the analysis buffer again
    // println!("Gathering input for analysis");    
    //TODO: implement input overlap again

    // Start analysis loop
    println!("Starting analysis");
    loop {
        // Aggregate all pending input
        let vec = buffer.take(resolution);
        // Apply fft and extract frequencies
        let fourier = fourier_analysis(&vec[..], &mut planner, Some(&mask));
        // Calculate dissonance of each note
        let scores = calculate_scores(fourier);
        // Display scores accordingly
        display_guitar(scores, notation);
    }
}

// fn display_controls() {
//     println!("[q] [←] [→] [l] [h]");
// }

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

const NOTE_NAMES_ENGLISH: [&str; 12] = [
    " C ", " C#", " D ", " D#", " E ", " F ", " F#", " G ", " G#", " A ", " A#", " B ",
];
const NOTE_NAMES_ROMANCE: [&str; 12] = [
    "Do ", "Do#", "Ré ", "Ré#", "Mi ", "Fa ", "Fa#", "Sol", "So#", "La ", "La#", "Si ",
];
const NOTE_COUNT: usize = 89;
const BASE_NOTE: usize = 12 * 4 + 10; // C1 + 4 octaves + 10 == A4
const BASE_FREQUENCY: f32 = 440f32; // Frequency of A4

fn calculate_scores(frequencies: Vec<Complex<f32>>) -> [f32; NOTE_COUNT] {
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
    // Normalize score
    for score in scores.iter_mut() {
        *score = (*score - min) / (max - min);
        *score = score.powf(0.5f32);
    }
    scores
}

const GUITAR_STRING_LENGTH: usize = 44;
const GUITAR_STRINGS: [usize; 6] = [16 + 0, 16 + 5, 16 + 10, 16 + 15, 16 + 19, 16 + 24];

fn display_guitar(scores: [f32; NOTE_COUNT], notation:[&str; 12]) {
    // Clear the terminal
    // crossterm::terminal::terminal().clear(crossterm::terminal::ClearType::All).unwrap();
    // Create buffer to avoid flicker
    let mut buffer = BufWriter::new(io::stdout());

    write!(&mut buffer, "\n 0 |").unwrap();

    for i in 1 .. GUITAR_STRING_LENGTH {
        write!(&mut buffer, "{:^3}", i).unwrap();
    }
    write!(&mut buffer, "\n").unwrap();

    // For every guitar strings
    for &j in GUITAR_STRINGS.iter().rev() {
        // For every note on that string
        for i in j..j + GUITAR_STRING_LENGTH {
            // Get note name and calculated score
            let name = notation[i % 12];
            let score = scores[i];
            // Write the name with the appropriate color
            let gradient = (clamp(score) * 255f32) as u8;
            write!(&mut buffer, "\x1b[30;48;2;{red};{green};{blue}m{name}",
                red = gradient,
                green = (255 - gradient),
                blue = gradient / 4,
                name = name
            ).unwrap();
            // Add the bar to differentiate the zero 'fret' from the rest
            if i == j {
                write!(&mut buffer, "\x1b[0;0m|").unwrap();
            }
        }
        writeln!(&mut buffer, "\x1b[0;0m").unwrap();
    }
    buffer.flush().unwrap();
}
