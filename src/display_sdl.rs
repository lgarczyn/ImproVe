// The SDL display loop

// Standard
use std::sync::mpsc::Receiver;

// Tools
use itertools::Itertools;
use palette::Gradient;
use palette::Hsv;
use palette::Srgb;

// Sdl
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

// Crate
use crate::display::DisplayOptions;
use crate::notes::Note::*;
use crate::scores::Scores;

// Guitar constants

// The note for every guitar strings, from E2 to E4
const STRING_COUNT: usize = 6;
const STRINGS: [usize; STRING_COUNT] = [
    E2 as usize,
    A2 as usize,
    D3 as usize,
    G3 as usize,
    B3 as usize,
    E4 as usize,
];

// Dimensions in pixels for every fretboard elements
const STRING_HEIGHT: u32 = 18;
const FRET_WIDTH: u32 = 27;
const FRET_COUNT: u32 = 44;
const FRET_LINE: u32 = 9;
const FONT_HEIGHT: u16 = STRING_HEIGHT as u16 - 1;

// Note range
const FIRST_NOTE: usize = STRINGS[0];
const LAST_NOTE: usize = STRINGS[STRING_COUNT - 1] + FRET_COUNT as usize;

// Font asset
const FONT_NAME: &str = "assets/UbuntuMono-R.ttf";

// Board graph window dimensions
const BOARD_HEIGHT: u32 = (STRING_COUNT as u32 + 1) * STRING_HEIGHT;
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
            texture_creator
                .create_texture_from_surface(&surface)
                .unwrap()
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
        draw_graph(&mut canvas_fourier, &scores);

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

    let note_scores = normalize(&scores.note_scores[FIRST_NOTE..LAST_NOTE]);
    let note_values = normalize(&scores.note_values[FIRST_NOTE..LAST_NOTE]);

    let gradient_score = {
        let gradient_a = Hsv::new(120.0, 1.0, 1.0);
        let gradient_b = Hsv::new(0.0, 1.0, 1.0);
        Gradient::new(vec![gradient_a, gradient_b])
    };
    // The canvas position
    let mut pnt = Point::new(0, 0);
    // Skip first line
    pnt = pnt.offset(0, STRING_HEIGHT as i32);

    // For every guitar strings
    for &j in STRINGS.iter().rev() {
        // For every note on that string
        for i in j..j + FRET_COUNT as usize {
            // Write the name with the appropriate color

            // Get note name and calculated score
            let texture = &texture_notes[i % 12];
            let score = note_scores[i - FIRST_NOTE];
            // Get the colored rectangle coordinates
            let rect = Rect::new(pnt.x, pnt.y, FRET_WIDTH, STRING_HEIGHT);
            // Get the gradient color
            let gradient_poll = gradient_score.get(score);
            let color: (u8, u8, u8) = Srgb::from(gradient_poll).into_format().into_components();
            // Draw tesxt and color to canvas
            canvas.set_draw_color(Color::from(color));
            canvas.fill_rect(rect).unwrap();
            canvas.copy(texture, None, Some(rect)).unwrap();
            
            // Underline notes being played (depending on value)
            
            // Get note value
            let value = note_values[i - FIRST_NOTE];
            // Get the colored rectangle coordinates
            let rect = Rect::new(pnt.x, pnt.y + STRING_HEIGHT as i32 - 1, FRET_WIDTH, 1);
            // Get the gradient color
            let color = (value * 255f32) as u8;
            let color: (u8, u8, u8) = (color, color, color);
            // Draw tesxt and color to canvas
            canvas.set_draw_color(Color::from(color));
            canvas.fill_rect(rect).unwrap();

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
use std::fmt::Debug;
use std::ops::Range;

// Map any numerical type from one range to another
// Doesn't actually guarantee the ranges will stay exclusive
// So use with care
fn map<F, T>(f: F, from: Range<F>, into: Range<T>, inv: bool) -> T
where
    F: ToPrimitive + Debug,
    T: ToPrimitive + NumCast + Debug,
{
    let fromf: Range<f64> = from.start.to_f64().unwrap()..from.end.to_f64().unwrap();
    let intof: Range<f64> = into.start.to_f64().unwrap()..into.end.to_f64().unwrap();
    let ff: f64 = f.to_f64().unwrap();

    let amplitude = (fromf.end - fromf.start) + std::f64::MIN_POSITIVE;
    let ratio = (ff - fromf.start) / amplitude;
    let mapped = ratio * (intof.end - intof.start);

    let mapped = if inv {
        intof.end - mapped
    } else {
        intof.start + mapped
    };
    match T::from(mapped.round()) {
        Some(mapped) => mapped,
        None => {
            eprintln!("could not cast {:?} from {:?} to {:?}", f, from, into);
            T::from(into.start).unwrap()
        }
    }
}

fn normalize(data:&[f32]) -> Vec<f32> {
    let (min, max) = data
        .iter()
        .cloned()
        .minmax()
        .into_option()
        .unwrap();

    data.iter().map(|&f| (f - min) / (max - min)).collect_vec()
}

fn draw_graph(canvas: &mut Canvas<Window>, scores: &Scores) {
    // Clear graph
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    draw_notes(canvas, scores);

    // Flush
    canvas.present();
}

// Display the fourier graph
#[allow(dead_code)]
fn draw_fourier(canvas: &mut Canvas<Window>, scores: &Scores) {
    canvas.set_draw_color(Color::RGB(30, 255, 30));

    // Skip boring frequencies
    let fourier = scores
        .fourier
        .iter()
        .cloned()
        .skip(30)
        .map(|mut f| {
            // boost large frequencies for prettier graph
            f.intensity = f.amplitude() * f.value;
            // change values to draw on a log scale
            f.value = f.value.ln();
            f
        })
        .collect_vec();

    // Get maximum frequency (alway the same)
    let min_hz = fourier.first().unwrap().value;
    let max_hz = fourier.last().unwrap().value;

    // Get maximum intensity (varies with time)
    let max_vo = fourier.iter().max().unwrap().intensity;

    // Draw uncorrected frequencies
    //let points = fourier
    //    .iter()
    //    .map(|f| {
    //        // apply reverse correction
    //        let i = f.intensity / crate::fourier::a_weigh_frequency(f.value.exp());
    //        Point::new(
    //            map(f.value, min_hz..max_hz, 0..FOURIER_WIDTH as i32 - 1, false),
    //            map(i, 0f32..max_vo, 0..FOURIER_HEIGHT as i32 - 1, true),
    //        )
    //    })
    //    .collect::<Vec<Point>>();

    //canvas.draw_lines(points.as_slice()).unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));

    // Draw corrected frequencies
    let points = fourier
        .iter()
        .map(|f| {
            Point::new(
                map(f.value, min_hz..max_hz, 0..FOURIER_WIDTH as i32 - 1, false),
                map(
                    f.intensity,
                    0f32..max_vo,
                    0..FOURIER_HEIGHT as i32 - 1,
                    true,
                ),
            )
        })
        .collect::<Vec<Point>>();

    canvas.draw_lines(points.as_slice()).unwrap();

    canvas.set_draw_color(Color::RGB(30, 30, 255));
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
pub fn draw_notes(canvas: &mut Canvas<Window>, scores: &Scores) {
    let (min, max) = scores.note_scores.iter().cloned().minmax().into_option().unwrap();

    let points = (0..FOURIER_WIDTH)
        .map(|x| {
            let i = map(x, 0..FOURIER_WIDTH, 0..scores.note_scores.len() - 1, false);
            let y = map(
                scores.note_scores[i],
                min..max,
                0..FOURIER_HEIGHT as i32 - 1,
                true,
            );
            Point::new(x as i32, y)
        })
        .collect_vec();

    canvas.set_draw_color(Color::RGB(255, 255, 255));

    canvas.draw_lines(points.as_slice()).unwrap();
}
