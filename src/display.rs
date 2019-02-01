use crate::notes::Note;

#[derive(Clone, Copy, Debug)]
pub enum Notation {
	English,
	Romance,
}

const NOTE_NAMES_ENGLISH: [&str; 12] = [
	" C ", " C#", " D ", " D#", " E ", " F ", " F#", " G ", " G#", " A ", " A#", " B ",
];
const NOTE_NAMES_ROMANCE: [&str; 12] = [
	"Do ", "Do#", "Ré ", "Ré#", "Mi ", "Fa ", "Fa#", "Sol", "So#", "La ", "La#", "Si ",
];

impl Notation {
	pub fn get_name(&self, note: Note) -> &str {
		let index = note.get_octave_index() as usize;
		self.get_names()[index]
	}
	pub fn get_names(&self) -> [&str; 12] {
		match &self {
			Notation::English => NOTE_NAMES_ENGLISH,
			Notation::Romance => NOTE_NAMES_ROMANCE,
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct DisplayOptions {
	pub notation: Notation,
	pub clear_term: bool,
	pub instrument: (),
}
