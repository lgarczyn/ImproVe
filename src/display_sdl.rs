// A wrapper for SDL display

use sdl2::Sdl;
use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::video::WindowPos;
use rustfft::num_complex::Complex;

use crate::scores::NOTE_COUNT;

pub struct DisplaySDL {
	canvas_fourier:WindowCanvas,
	canvas_board:WindowCanvas,
	options:crate::display::DisplayOptions,
}

const FOURIER_HEIGHT:u32 = 200;
const FOURIER_WIDTH:u32 = 1024;

const STRINGS: [usize; 6] = [16 + 0, 16 + 5, 16 + 10, 16 + 15, 16 + 19, 16 + 24];

const STRING_HEIGHT:u32 = 18;
const STRING_COUNT:u32 = 6;
const FRET_WIDTH:u32 = 27;
const FRET_COUNT:u32 = 44;
const FRET_LINE:u32 = 5;

const SQUARE:(u32, u32) = (FRET_WIDTH, STRING_HEIGHT);

const BOARD_HEIGHT:u32 = (STRING_COUNT + 1) * STRING_HEIGHT;
const BOARD_WIDTH:u32 = (FRET_COUNT) * FRET_WIDTH + FRET_LINE;

impl DisplaySDL {

	// Create new SDL window
	pub fn new(sdl:&Sdl, options:crate::display::DisplayOptions) -> DisplaySDL {
    	let video_subsystem = sdl.video().unwrap();
 
    	let mut window = video_subsystem.window("ImproVe Fourier", FOURIER_WIDTH, FOURIER_HEIGHT)
			.position_centered()
			.build()
			.unwrap();
		let pos = window.position();
		window.set_position(
			WindowPos::Centered,
			WindowPos::Positioned(pos.1 - BOARD_HEIGHT as i32 - 100));
 
    	let canvas_fourier = window.into_canvas().build().unwrap();
 
    	let window = video_subsystem.window("ImproVe Fretboard", BOARD_WIDTH, BOARD_HEIGHT)
			.position_centered()
			.build()
			.unwrap();
 
    	let canvas_board = window.into_canvas().build().unwrap();

		DisplaySDL {
			canvas_fourier,
			canvas_board,
			options
		}
	}

	pub fn draw_fourier(&mut self, fourier:&Vec<Complex<f32>>) {

		let canvas = &mut self.canvas_fourier;

		canvas.set_draw_color(Color::RGB(0, 0, 0));

		canvas.clear();

		canvas.set_draw_color(Color::RGB(255, 255, 255));

		let max_hz = fourier.last().unwrap().re;
		let max_vo = fourier.iter().max_by(|a, b|
			a.im.partial_cmp(&b.im).unwrap()
		).unwrap().im;

		canvas.draw_points(fourier.iter().map(|c|
			Point::new(
				(c.re / max_hz * FOURIER_WIDTH as f32) as i32 + 1,
				FOURIER_HEIGHT as i32 - 1 - (c.im / max_vo * (FOURIER_HEIGHT - 1) as f32) as i32,
			)
		).collect::<Vec<Point>>().as_slice()).unwrap();

		canvas.present();
	}

	pub fn draw_notes(&mut self, scores:&[f32; NOTE_COUNT]) {

		let canvas = &mut self.canvas_board;

		canvas.set_draw_color(Color::RGB(0, 0, 0));
		canvas.clear();

		let mut pnt = Point::new(0, 0);

		// Display the fret count
		//write!(&mut buffer, " 0 |").unwrap();
		//for i in 1 .. GUITAR_STRING_LENGTH {
		//	write!(&mut buffer, "{:^3}", i).unwrap();
		//}
		//write!(&mut buffer, "\n").unwrap();
		pnt = pnt.offset(0, STRING_HEIGHT as i32);

		// For every guitar strings
		for &j in STRINGS.iter().rev() {
			// For every note on that string
			for i in j..j + FRET_COUNT as usize {
				// Get note name and calculated score
				let name = self.options.notation.get_names()[i % 12];
				let score = scores[i];
				// Write the name with the appropriate color
            	let score = score.max(0f32).min(1f32);
				let gradient = (score * 255f32) as u8;
				canvas.set_draw_color(Color::RGB(gradient, 255 - gradient, gradient / 4));
				canvas.fill_rect(Rect::new(pnt.x, pnt.y, FRET_WIDTH, STRING_HEIGHT)).unwrap();
				// Add the bar to differentiate the zero 'fret' from the rest
				if i == j {
					pnt = pnt.offset(FRET_LINE as i32, 0);
				}
				pnt = pnt.offset(FRET_WIDTH as i32, 0);
			}
			pnt = Point::new(0, pnt.y() + STRING_HEIGHT as i32);
		}
		canvas.present();
	}
}
