// Standard
use std::sync::mpsc::{channel, Sender};

// Parser
use clap::{Arg, App};

// SDL2
use sdl2::audio::{AudioCallback, AudioSpecDesired};

// Crate
mod dissonance;
mod audio_buffer;
mod fourier;
mod scores;
mod display;
mod display_sdl;
mod display_term;

use self::display::DisplayOptions;
use self::audio_buffer::{AudioBuffer, BufferOptions};
use self::scores::Scores;

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
        .arg(Arg::with_name("discard")
            .short("d")
            .long("discard")
            .help("Allows the program to discard data if latency is too high\n"))
        .arg(Arg::with_name("overlap")
            .short("o")
            .long("overlap")
            .help("Allows the program to reuse data if the latency is too low\n"))
        .arg(Arg::with_name("terminal")
            .short("t")
            .long("terminal")
            .help("Use the terminal instead of SDL2 windows\n"))
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

    // The channel to get data from audio callback and back
    let (audio_sender, audio_receiver) = channel::<Vec<f32>>();
    let (score_sender, score_receiver) = channel::<Scores>();

    // Get the SDL objects
    let sdl_context = sdl2::init()?;
    let audio_subsystem = sdl_context.audio()?;
    println!("Capture Driver = {}", audio_subsystem.current_audio_driver());
    println!("Capture Spec = {:?}", audio_subsystem.audio_playback_device_name(0));

    // Set the desired specs
    let desired_spec = AudioSpecDesired {
        freq: Some(88200),
        channels: Some(1),
        samples: None
    };

    // Build the callback object and start recording
    let mut received_spec = None;

    let capture_device = audio_subsystem.open_capture(None, &desired_spec, |spec| {
        println!("Capture Spec = {:?}", spec);
        received_spec = Some(spec);
        Recorder { audio_sender }
    })?;
    let freq = received_spec.unwrap().freq;

    capture_device.resume();

    // Build audio receiver and aggrgator
    let buffer = AudioBuffer::new(audio_receiver, buf_opt);

    // Start the data analysis
    std::thread::spawn(move || {
        fourier::fourier_thread(buffer, score_sender, freq);
    });

    if matches.is_present("terminal") {
        return display_term::display(score_receiver, disp_opt);
    } else {
        return display_sdl::display(sdl_context, score_receiver, disp_opt);
    }
}

// Audio callback object, simply allocates and transfers to a sender
struct Recorder {
    audio_sender: Sender<Vec<f32>>,
}

impl AudioCallback for Recorder {
    type Channel = f32;

    fn callback(&mut self, input: &mut [f32]) {
        self.audio_sender.send(input.to_owned()).ok();
    }
}


