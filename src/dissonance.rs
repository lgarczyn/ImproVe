
/*

Dissonance.rs helps calculating the dissonance between an array of frequency-intensity couples
and a virtual idealized note played on a harmonic-rich instrument.

Currently it approximates the dissonance between a single frequency and an instrument.
The lookup table was extracted from a graph, and is therefore less than accurate.

Further improvements include a more scientific data source, varying instruments.

*/

// Plomp-Levelt dissonance formula
// Calculates the perceived dissonance between two pure frequencies
// source: https://books.google.fr/books?id=W2_n1R5F2XoC&lpg=PA202&ots=Pp8UydRXiK&dq=%22plomb-levelt%22%20curve%20formula&pg=PA202#v=onepage&q&f=false

pub fn dissonance(f_1:f32, f_2:f32, ) -> f32 {

    //dbg!((f_1, a_1, f_2, a_2));
    const A:f32 = 3.5;
    const B:f32 = 5.75;
    const D_S:f32 = 0.24;
    const S1:f32 = 0.021;
    const S2:f32 = 19.0;

    let s = D_S / (S1 * f_1.min(f_2) + S2);

    let exp = (f_2 - f_1).abs() * s;

    // Find out why curve maximum doesn't reach 1.0 without fix
    (consts::E.powf(-A * exp) - consts::E.powf(-B * exp))
}

// Calculates the dissonance between two frequencies
// Currently doesn't differentiates between pitch of inverse ratios
pub fn estimate(f_heard: f32, f_inst: f32) -> f32 {
    
    LOOKUP_CACHE.with(|f| {
        let mut map = f.borrow_mut();
        let cache = get_map_pos(&mut map, f_heard, f_inst);
        if let Some(cached) = cache {
            return *cached;
        }
        let mut dis = 0f32;
        dis += dissonance(f_heard, f_inst);
            
        for i in 2 .. 32 {
            let harmonic_freq_high = f_inst * i as f32;
            let harmonic_freq_low = f_inst / i as f32;
            let harmonic_int = 1.3f32.powi(1 - i);
            let harmonic_dis =
                dissonance(f_heard, harmonic_freq_high)
                + dissonance(f_heard, harmonic_freq_low);
            dis += harmonic_dis * harmonic_int;
        }

        *cache = Some(dis);

        //panic!();
        dis
    })
}

// Because the formula is horribly slow and has to be run on n*m frequencies
// It is cached into the worst data structure ever created
// A 2-dimensional log-scaled lookup table
// It runs somewhat fast though

use std::f32::consts;
use std::cell::RefCell;

type Map = Vec<Vec<Option<f32>>>;
pub const RESOLUTION:f32 = 1000f32;
pub const MIN_FREQ:f32 = 0.0001f32;

thread_local! {
    pub static LOOKUP_CACHE: RefCell<Map> = RefCell::new(Map::new());
}

fn get_index(freq:f32) -> usize {
    let ln = freq.max(MIN_FREQ).ln() * RESOLUTION;
    let lf = ln - (MIN_FREQ.ln() * RESOLUTION);
    lf as usize
}

fn get_map_pos<'a>(map:&'a mut Map, f_1:f32, f_2:f32) -> &'a mut Option<f32> {
    let i_1 = get_index(f_1);
    let i_2 = get_index(f_2);

    if i_1 >= map.len() {
        map.resize(i_1 + 1, vec![None; i_2 + 1]);
    }

    if i_2 >= map[i_1].len() {
        map[i_1].resize(i_2 + 1, None);
    }
    &mut map[i_1][i_2]
}