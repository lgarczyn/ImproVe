// The terminal display loop

use crate::display::DisplayOptions;
use crate::notes::Note::*;
use crate::notes::{Note, NOTE_COUNT};
use crate::scores::Scores;

use std::io;
use std::io::BufWriter;
use std::io::Write;
use std::sync::mpsc::Receiver;

// Number of notes in line
const GUITAR_STRING_LENGTH: usize = 44;
// Every string defined by their note (E2 to E4)
const GUITAR_STRINGS: [Note; 6] = [E2, A2, D3, G3, B3, E4];

// Clear terminal and display guitar
fn guitar(scores: &[f32; NOTE_COUNT], options: DisplayOptions) {
    // Create buffer to avoid flicker
    let mut buffer = BufWriter::new(io::stdout());

    // Add the clear screen message to the buffer
    if options.clear_term {
        write!(
            &mut buffer,
            "{}",
            termion::cursor::Up(GUITAR_STRINGS.len() as u16 + 1)
        )
        .unwrap();
    }

    // Display the fret count
    write!(&mut buffer, " 0 |").unwrap();
    for i in 1..GUITAR_STRING_LENGTH {
        write!(&mut buffer, "{:^3}", i).unwrap();
    }
    writeln!(&mut buffer).unwrap();

    // For every guitar strings
    for &string in GUITAR_STRINGS.iter().rev() {
        // For every note on that string
        for note in string.iter_from().take(GUITAR_STRING_LENGTH) {
            // Get note name and calculated score
            let name = options.notation.get_name(note);
            let score = scores[note as usize];
            let score = score.max(0f32).min(1f32);
            // Write the name with the appropriate color
            let gradient = (score * 255f32) as u8;
            write!(
                &mut buffer,
                "\x1b[30;48;2;{red};{green};{blue}m{name}",
                red = gradient,
                green = (255 - gradient),
                blue = gradient / 4,
                name = name
            )
            .unwrap();
            // Add the bar to differentiate the zero 'fret' from the rest
            if string == note {
                write!(&mut buffer, "\x1b[0;0m|").unwrap();
            }
        }
        writeln!(&mut buffer, "\x1b[0;0m").unwrap();
    }
    buffer.flush().unwrap();
}

// Simply feeds the scores into the guitar display
pub fn display(receiver: Receiver<Scores>, options: DisplayOptions) -> Result<(), String> {
    for scores in receiver.into_iter() {
        guitar(&scores.note_scores, options);
    }
    Ok(())
}
