// The SDL display loop

use std::sync::mpsc::Receiver;

use rustfft::num_complex::Complex;

use itertools::Itertools;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::render::Texture;
use sdl2::video::Window;
use sdl2::video::WindowPos;
use sdl2::Sdl;

use crate::display::DisplayOptions;
use crate::scores::Scores;

// Guitar constants

// The note for every guitar strings, from E2 to E4
const STRINGS: [usize; 6] = [16 + 0, 16 + 5, 16 + 10, 16 + 15, 16 + 19, 16 + 24];

// Dimensions in pixels for every fretboard elements
const STRING_HEIGHT: u32 = 18;
const STRING_COUNT: u32 = 6;
const FRET_WIDTH: u32 = 27;
const FRET_COUNT: u32 = 44;
const FRET_LINE: u32 = 9;
const FONT_HEIGHT: u16 = STRING_HEIGHT as u16 - 1;

// Font asset
const FONT_NAME: &str = "assets/UbuntuMono-R.ttf";

// Board graph window dimensions
const BOARD_HEIGHT: u32 = (STRING_COUNT + 1) * STRING_HEIGHT;
const BOARD_WIDTH: u32 = (FRET_COUNT) * FRET_WIDTH + FRET_LINE;

// Fourier graph dimensions
const FOURIER_HEIGHT: u32 = 200;
const FOURIER_WIDTH: u32 = 1024;

// The display loop, receives data from the fourier thread
// Hard to abstract further because of rust-sdl safety guards
pub fn display(
	sdl: Sdl,
	receiver: Receiver<Scores>,
	options: DisplayOptions,
) -> Result<(), String> {
	// Open windows

	let video_subsystem = sdl.video().unwrap();

	let mut window = video_subsystem
		.window("ImproVe Fourier", FOURIER_WIDTH, FOURIER_HEIGHT)
		.position_centered()
		.build()
		.unwrap();
	let pos = window.position();
	window.set_position(
		WindowPos::Centered,
		WindowPos::Positioned(pos.1 - BOARD_HEIGHT as i32 - 100),
	);

	let mut canvas_fourier = window.into_canvas().build().unwrap();

	let window = video_subsystem
		.window("ImproVe Fretboard", BOARD_WIDTH, BOARD_HEIGHT)
		.position_centered()
		.build()
		.unwrap();

	let mut canvas_board = window.into_canvas().build().unwrap();

	canvas_board.present();
	canvas_fourier.present();

	// Build text textures, for use in the loop

	// Init the front
	let ttf_context = sdl2::ttf::init().unwrap();
	let texture_creator = canvas_board.texture_creator();
	let font = ttf_context.load_font(FONT_NAME, FONT_HEIGHT).unwrap();

	// Build a texture for every note names
	let textures = options
		.notation
		.get_names()
		.iter()
		.map(|name| {
			let surface = font
				.render(name)
				.blended(Color::RGBA(30, 30, 30, 255))
				.unwrap();
			let texture = texture_creator
				.create_texture_from_surface(&surface)
				.unwrap();
			texture
		})
		.collect_vec();

	// Build the header, with numbers from 0 to 43, but with an additional space between 0 and 1
	let header = std::iter::once(" 0  ".to_string())
		.chain((1..FRET_COUNT).map(|i| format!("{:^3}", i)))
		.join("");

	let surface_header = font
		.render(&header)
		.blended(Color::RGB(255, 255, 255))
		.unwrap();

	let texture_header = texture_creator
		.create_texture_from_surface(&surface_header)
		.unwrap();

	// Build the event pump, to kill everything elegantly
	let mut events = sdl.event_pump().unwrap();

	// Iterate on scores
	for scores in receiver.into_iter() {
		// Draw the fourier frequency graph
		draw_fourier(&mut canvas_fourier, &scores);

		// Draw the fretboard graph
		draw_board(&mut canvas_board, &scores, &textures, &texture_header);

		for event in events.poll_iter() {
			match event {
				Event::Quit { .. }
				| Event::KeyDown {
					keycode: Some(Keycode::Escape),
					..
				} => {
					return Ok(());
				}
				_ => {}
			}
		}
	}

	Ok(())
}

// Display the fretboard graph
fn draw_board(
	canvas: &mut Canvas<Window>,
	scores: &Scores,
	texture_notes: &[Texture],
	texture_header: &Texture,
) {
	// Clear canvas
	canvas.set_draw_color(Color::RGB(30, 30, 30));
	canvas.clear();

	// Display Header
	canvas
		.copy(
			&texture_header,
			None,
			Some(Rect::new(0, 0, BOARD_WIDTH, STRING_HEIGHT)),
		)
		.unwrap();

	// The canvas position
	let mut pnt = Point::new(0, 0);

	pnt = pnt.offset(0, STRING_HEIGHT as i32);

	// For every guitar strings
	for &j in STRINGS.iter().rev() {
		// For every note on that string
		for i in j..j + FRET_COUNT as usize {
			// Get note name and calculated score
			let texture = &texture_notes[i % 12];
			let score = scores.notes[i];
			// Write the name with the appropriate color
			let score = score.max(0f32).min(1f32);
			let gradient = (score * 255f32) as u8;
			let rect = Rect::new(pnt.x, pnt.y, FRET_WIDTH, STRING_HEIGHT);
			canvas.set_draw_color(Color::RGB(gradient, 255 - gradient, gradient / 4));
			canvas.fill_rect(rect).unwrap();
			canvas.copy(texture, None, Some(rect)).unwrap();
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

use num_traits::cast::{NumCast, ToPrimitive};
use std::ops::Range;

fn map<F, T>(f: F, from: Range<F>, into: Range<T>, inv: bool) -> T
where
	F: ToPrimitive,
	T: ToPrimitive + NumCast,
{
	let from: Range<f64> = from.start.to_f64().unwrap()..from.end.to_f64().unwrap();
	let into: Range<f64> = into.start.to_f64().unwrap()..into.end.to_f64().unwrap();
	let f: f64 = f.to_f64().unwrap();

	let ratio = (f - from.start) / (from.end - from.start);
	let mapped = ratio * (into.end - into.start);

	let mapped = if inv {
		into.end - mapped
	} else {
		into.start + mapped
	};
	T::from(mapped).unwrap()
}

// Display the fourier graph
fn draw_fourier(canvas: &mut Canvas<Window>, scores: &Scores) {
	// Clear graph
	canvas.set_draw_color(Color::RGB(0, 0, 0));

	canvas.clear();

	canvas.set_draw_color(Color::RGB(30, 255, 30));

	// Get maximum frequency (alway the same)
	let max_hz = scores.fourier.last().unwrap().re;

	// Get maximum intensity (varies with time)
	let max_vo = scores
		.fourier
		.iter()
		.max_by(|a, b| a.im.partial_cmp(&b.im).unwrap())
		.unwrap()
		.im;

	// Draw uncorrected frequencies
	let points = scores
		.fourier
		.iter()
		.map(|c| {
			let im = c.im / crate::fourier::a_weigh_frequency(c.re);
			Point::new(
				map(c.re, 0f32..max_hz, 0..FOURIER_WIDTH as i32 - 1, false),
				map(im, 0f32..max_vo, 0..FOURIER_HEIGHT as i32 - 1, true),
			)
		})
		.collect::<Vec<Point>>();

	canvas.draw_lines(points.as_slice()).unwrap();

	canvas.set_draw_color(Color::RGB(255, 255, 255));

	// Draw corrected frequenciesß
	let points = scores
		.fourier
		.iter()
		.map(|c| {
			Point::new(
				map(c.re, 0f32..max_hz, 0..FOURIER_WIDTH as i32 - 1, false),
				map(c.im, 0f32..max_vo, 0..FOURIER_HEIGHT as i32 - 1, true),
			)
		})
		.collect::<Vec<Point>>();

	canvas.draw_lines(points.as_slice()).unwrap();

	canvas.set_draw_color(Color::RGB(30, 30, 255));

	// Draw scale
	let points = (0..20)
		.map(|i| Point::new((FOURIER_WIDTH / 2u32.pow(i)) as i32, 0))
		.collect::<Vec<Point>>();

	canvas.draw_lines(points.as_slice()).unwrap();

	canvas.set_draw_color(Color::RGB(255, 255, 255));

	draw_major_chord_graph(canvas, scores);

	// Flush
	canvas.present();
}

// Various graphs, mostly for debugging.

#[allow(dead_code)]
pub fn draw_pure_dissonance_graph(canvas: &mut Canvas<Window>, _: &Scores) {
	// Draw data points
	let points = [50f32, 100f32, 200f32, 400f32, 800f32, 1600f32]
		.iter()
		.cloned()
		.map(|f_1| {
			(0..FOURIER_WIDTH as i32)
				.zip(std::iter::repeat(f_1))
				.map(|(x, f_1)| {
					let factor = 3f32 * x as f32 / FOURIER_WIDTH as f32;
					let yf = 1f32 - crate::dissonance::dissonance(f_1, f_1 * factor);
					let y = (yf * FOURIER_HEIGHT as f32) as i32;
					Point::new(x, y)
				})
		})
		.flatten()
		.collect_vec();

	for (points, i) in points
		.iter()
		.chunks(FOURIER_WIDTH as usize)
		.into_iter()
		.zip((0..255).step_by(255 / 6))
	{
		canvas.set_draw_color(Color::RGB(i, 255 - i, 126));
		let points = points.cloned().collect_vec();
		canvas.draw_lines(points.as_slice()).unwrap();
	}
}

#[allow(dead_code)]
pub fn draw_major_chord_graph(canvas: &mut Canvas<Window>, _: &Scores) {
	let points = (0..FOURIER_WIDTH as i32)
		.map(|x| {
			let factor = 9f32 * x as f32 / FOURIER_WIDTH as f32;
			let yf = 1f32 - (crate::dissonance::estimate(440f32, 440f32 * factor)) * 2f32;
			let y = (yf * FOURIER_HEIGHT as f32) as i32;
			Point::new(x, y)
		})
		.collect_vec();

	canvas.set_draw_color(Color::RGB(255, 255, 255));

	canvas.draw_lines(points.as_slice()).unwrap();

	let points = (0..FOURIER_WIDTH as i32)
		.map(|x| {
			let factor = 9f32 * x as f32 / FOURIER_WIDTH as f32;
			let yf = 1f32
				- (crate::dissonance::estimate(
					crate::notes::Note::C4.freq(),
					crate::notes::Note::C4.freq() * factor,
				)) * 2f32;
			let y = (yf * FOURIER_HEIGHT as f32) as i32;
			Point::new(x, y)
		})
		.collect_vec();

	canvas.set_draw_color(Color::RGB(255, 0, 255));

	canvas.draw_lines(points.as_slice()).unwrap();

	let points = (0..FOURIER_WIDTH as i32)
		.map(|x| {
			let factor = 9f32 * x as f32 / FOURIER_WIDTH as f32;
			let yf = 1f32
				- (crate::dissonance::estimate(
					crate::notes::Note::A5.freq(),
					crate::notes::Note::A5.freq() * factor,
				)) * 2f32;
			let y = (yf * FOURIER_HEIGHT as f32) as i32;
			Point::new(x, y)
		})
		.collect_vec();

	canvas.set_draw_color(Color::RGB(255, 255, 0));

	canvas.draw_lines(points.as_slice()).unwrap();
}

#[allow(dead_code)]
pub fn draw_score_octave_graph(canvas: &mut Canvas<Window>, scores: &Scores) {
	let notes = (0..FOURIER_WIDTH)
		.map(|x| {
			let hz = crate::notes::Note::E3.freq() * 2f32.powf(x as f32 / FOURIER_WIDTH as f32);
			let mut score = 0f32;
			for &Complex { re: a, im: b } in scores.fourier.iter() {
				if b > 0.1f32 {
					score += crate::dissonance::estimate(a, hz) * b;
				}
			}
			score
		})
		.collect_vec();

	let mut min = std::f32::INFINITY;
	let mut max = std::f32::NEG_INFINITY;

	for &score in notes.iter() {
		min = min.min(score);
		max = max.max(score);
	}

	let points = notes
		.into_iter()
		.enumerate()
		.map(|(x, s)| Point::new(x as i32, map(s, min..max, 0..FOURIER_HEIGHT as i32, true)))
		.collect_vec();

	canvas.set_draw_color(Color::RGB(255, 255, 0));

	canvas.draw_lines(points.as_slice()).unwrap();
}

#[allow(dead_code)]
pub fn draw_freq_map(canvas: &mut Canvas<Window>, _scores: &Scores) {
	for x in 0..FOURIER_WIDTH {
		for y in 0..FOURIER_HEIGHT {
			let xf = map(
				x,
				0..FOURIER_WIDTH,
				crate::notes::Note::E1.freq()..crate::notes::Note::E8.freq(),
				false,
			);
			let yf = map(
				y,
				0..FOURIER_HEIGHT,
				crate::notes::Note::E1.freq()..crate::notes::Note::E8.freq(),
				false,
			);
			let dis = crate::dissonance::estimate(xf, yf);
			let i = map(dis, 0f32..1f32, 0..255u32, false) as u8;
			canvas.set_draw_color(Color::RGB(i, 255 - i, 126));
			canvas.draw_point(Point::new(x as _, y as _)).unwrap();
		}
	}
}
