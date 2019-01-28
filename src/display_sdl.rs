// A wrapper for SDL display

use sdl2::Sdl;
use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use rustfft::num_complex::Complex;

pub struct DisplaySDL {
	sdl_context:Sdl,
	canvas:Option<WindowCanvas>,
}

const FOURIER_HEIGHT:u32 = 200;
const FOURIER_WIDTH:u32 = 1024;

impl DisplaySDL {

	// Create new SDL window
	pub fn new() -> DisplaySDL {
		let sdl_context = sdl2::init().unwrap();
    	let video_subsystem = sdl_context.video().unwrap();
 
    	let window = video_subsystem.window("ImproVe Fourier", FOURIER_WIDTH, FOURIER_HEIGHT)
        .position_centered()
        .build()
        .unwrap();
 
    	let canvas = window.into_canvas().build().unwrap();

		DisplaySDL {
			sdl_context,
			canvas: Some(canvas),
		}
	}

	pub fn draw_fourier(&mut self, fourier:&Vec<Complex<f32>>) {

		if let Some(canvas) = &mut self.canvas {
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

		let mut event_pump = self.sdl_context.event_pump().unwrap();
		for event in event_pump.poll_iter() {
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					self.canvas = None;
				},
				_ => {}
			}
		}
	}
}
