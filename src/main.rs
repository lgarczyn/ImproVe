// Standard
use std::sync::mpsc::channel;
use std::thread;
use std::vec;
use std::io::stdin;

// Tools
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

// Project
mod dissonance;
mod audio_buffer;
mod display;
mod display_sdl;
mod scores;
use self::audio_buffer::{AudioBuffer, BufferOptions};
use self::display::{guitar, DisplayOptions};

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
                Ok(32 ..= 1048576) => Ok(()),
                Ok(_) => Err("Argument out of range: (32 .. 1048576)".to_owned()),
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
        .arg(Arg::with_name("discard")
            .short("d")
            .long("discard")
            .help("Allows the program to discard data if latency is too high\n"))
        .arg(Arg::with_name("overlap")
            .short("o")
            .long("overlap")
            .help("Allows the program to reuse data if the latency is too low\n"))
        .arg(Arg::with_name("noclear")
            .short("c")
            .long("noclear")
            .help("Prevents the program from attempting to clear the terminal\n"))
        .get_matches();
    // Get notation convention
    let notation = match matches.value_of("notation").unwrap()
    {
        "e" => display::Notation::English,
        _ => display::Notation::Romance,
    };
    // Get display options object
    let disp_opt = DisplayOptions{
        notation,
        clear_term:!matches.is_present("noclear"),
        instrument:()
    };

    let mut buf_opt = BufferOptions::default();

    // Get number of values to read in a single FFT
    buf_opt.resolution = matches.value_of("resolution").unwrap().parse::<usize>().unwrap();
    // Check if values can be discarded if input is too fast
    buf_opt.discard = matches.is_present("discard");
    // Check if values can be analyzed multiple times if input is too slow
    buf_opt.overlap = matches.is_present("overlap");

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
    // The audio buffer to get the audio data in appropriately sized packets
    let buffer = AudioBuffer::new(receiver, buf_opt);

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

    fourier_thread(buffer, disp_opt);
}

// Receives audio input, start FFT on most recent data and display results
fn fourier_thread(
    buffer:AudioBuffer,
    display_options:DisplayOptions) {
    // The FFT pool, allows for optimized yet flexible data sizes
    let mut planner = FFTplanner::<f32>::new(false);
    // The audio buffer, to get uniformly-sized audio packets
    let mut buffer = buffer;

    // Get the first first few seconds of recording
    println!("Gathering noise profile");
    let vec = buffer.take();
    // Extract frequencies to serve as mask
    let mask = fourier_analysis(&vec[..], &mut planner, None);

    let mut drawer = display_sdl::DisplaySDL::new();

    // Start analysis loop
    println!("Starting analysis");
    loop {
        // Aggregate all pending input
        let vec = buffer.take();
        // Apply fft and extract frequencies
        let fourier = fourier_analysis(&vec[..], &mut planner, Some(&mask));
        drawer.draw_fourier(&fourier);
        // Calculate dissonance of each note
        let scores = scores::calculate(fourier);
        // Display scores accordingly
        guitar(scores, display_options);
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

