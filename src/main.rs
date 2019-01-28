// Standard
use std::sync::mpsc::{channel, Sender};
use std::vec;

// Tools
use itertools::Itertools;

// Math
use rustfft::num_complex::Complex;
use rustfft::FFTplanner;

// Parser
use clap::{Arg, App};

// SDL2
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::Sdl;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

// Project
mod dissonance;
mod audio_buffer;
mod display;
mod display_sdl;
mod scores;
use self::audio_buffer::{AudioBuffer, BufferOptions};
use self::display::{guitar, DisplayOptions};
use self::display_sdl::DisplaySDL;

fn main() -> Result<(), String> {
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
    // Get display option
    let disp_opt = DisplayOptions{
        notation,
        clear_term:!matches.is_present("noclear"),
        instrument:()
    };

    // Get audio buffering options
    let mut buf_opt = BufferOptions::default();
    // Get number of values to read in a single FFT
    buf_opt.resolution = matches.value_of("resolution").unwrap().parse::<usize>().unwrap();
    // Check if values can be discarded if input is too fast
    buf_opt.discard = matches.is_present("discard");
    // Check if values can be analyzed multiple times if input is too slow
    buf_opt.overlap = matches.is_present("overlap");

    // The channel to get data from audio callback
    let (sender, receiver) = channel::<Vec<f32>>();

    // Get the SDL objects
    let sdl_context = sdl2::init()?;
    let audio_subsystem = sdl_context.audio()?;

    // Set the desired specs
    let desired_spec = AudioSpecDesired {
        freq: None,
        channels: Some(1),
        samples: None
    };

    // Build the callback object and start recording
    let capture_device = audio_subsystem.open_capture(None, &desired_spec, |spec| {
        println!("Capture Spec = {:?}", spec);
        Recorder { sender }
    })?;

    capture_device.resume();

    // Build audio receiver and aggrgator
    let buffer = AudioBuffer::new(receiver, buf_opt);

    // Start the data analysis
    fourier_thread(buffer, sdl_context, disp_opt);
    Ok(())
}

// Audio callback object, simply allocates and transfers to a sender
struct Recorder {
    sender: Sender<Vec<f32>>,
}

impl AudioCallback for Recorder {
    type Channel = f32;

    fn callback(&mut self, input: &mut [f32]) {
        self.sender.send(input.to_owned()).unwrap();
    }
}

// Receives audio input, start FFT on most recent data and display results
fn fourier_thread(
    buffer:AudioBuffer,
    sdl:Sdl,
    display_options:DisplayOptions) {
    // The FFT pool, allows for optimized yet flexible data sizes
    let mut planner = FFTplanner::<f32>::new(false);
    // The audio buffer, to get uniformly-sized audio packets
    let mut buffer = buffer;
    // The SDL window wrapper to draw stuff
    let mut disp = DisplaySDL::new(&sdl);

    // Get the first first few seconds of recording
    println!("Gathering noise profile");
    let vec = buffer.take();
    // Extract frequencies to serve as mask
    let mask = fourier_analysis(&vec[..], &mut planner, None);

    let mut events = sdl.event_pump().unwrap();

    // Start analysis loop
    println!("Starting analysis");
    loop {
        // Aggregate all pending input
        let vec = buffer.take();
        // Apply fft and extract frequencies
        let fourier = fourier_analysis(&vec[..], &mut planner, Some(&mask));
        // Draw the fourier transform for info.
        disp.draw_fourier(&fourier);
        // Calculate dissonance of each note
        let scores = scores::calculate(fourier);
        // Display scores accordingly
        guitar(scores, display_options);

        // Check for quit events
		for event in events.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					return;
				},
				_ => {}
			}
		}
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

