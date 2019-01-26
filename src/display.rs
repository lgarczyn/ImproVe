use crate::scores::NOTE_COUNT;
use std::io::BufWriter;
use std::io::Write;
use std::io;
use clampf::clamp;

#[derive(Clone, Copy, Debug)]
pub enum Notation {
	English,
	Romance
}

const NOTE_NAMES_ENGLISH: [&str; 12] = [
    " C ", " C#", " D ", " D#", " E ", " F ", " F#", " G ", " G#", " A ", " A#", " B ",
];
const NOTE_NAMES_ROMANCE: [&str; 12] = [
    "Do ", "Do#", "Ré ", "Ré#", "Mi ", "Fa ", "Fa#", "Sol", "So#", "La ", "La#", "Si ",
];

impl Notation {
	pub fn get_names(&self) -> [&str; 12] {
		match &self {
			Notation::English => NOTE_NAMES_ENGLISH,
			Notation::Romance => NOTE_NAMES_ROMANCE
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct DisplayOptions {
	pub notation:Notation,
	pub clear_term:bool,
	pub instrument:()
}

const GUITAR_STRING_LENGTH: usize = 44;
const GUITAR_STRINGS: [usize; 6] = [16 + 0, 16 + 5, 16 + 10, 16 + 15, 16 + 19, 16 + 24];

pub fn guitar(scores: [f32; NOTE_COUNT], options:DisplayOptions) {
    // Clear the terminal
    // crossterm::terminal::terminal().clear(crossterm::terminal::ClearType::All).unwrap();
    // Create buffer to avoid flicker
    let mut buffer = BufWriter::new(io::stdout());

    // Add the clear screen message to the buffer
	if options.clear_term {
    	print!("{}[2J\n", 27 as char);
	}

    // Display the fret count
    write!(&mut buffer, " 0 |").unwrap();
    for i in 1 .. GUITAR_STRING_LENGTH {
        write!(&mut buffer, "{:^3}", i).unwrap();
    }
    write!(&mut buffer, "\n").unwrap();

    // For every guitar strings
    for &j in GUITAR_STRINGS.iter().rev() {
        // For every note on that string
        for i in j..j + GUITAR_STRING_LENGTH {
            // Get note name and calculated score
            let name = options.notation.get_names()[i % 12];
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
