
/*

Dissonance.rs helps calculating the dissonance between an array of frequency-intensity couples
and a virtual idealized note played on a harmonic-rich instrument.

Currently it approximates the dissonance between a single frequency and an instrument.
The lookup table was extracted from a graph, and is therefore less than accurate.

Further improvements include a more scientific data source, varying instruments.

*/

use std::f32::consts;

// Plomp-Levelt dissonance formula
// Calculates the perceived dissonance between two pure frequencies
// source: https://books.google.fr/books?id=W2_n1R5F2XoC&lpg=PA202&ots=Pp8UydRXiK&dq=%22plomb-levelt%22%20curve%20formula&pg=PA202#v=onepage&q&f=false

pub fn dissonance(f_1:f32, a_1:f32, f_2:f32, a_2:f32) -> f32 {

    //dbg!((f_1, a_1, f_2, a_2));
    const A:f32 = 3.5;
    const B:f32 = 5.75;
    const D_S:f32 = 0.24;
    const S1:f32 = 0.021;
    const S2:f32 = 19.0;

    let s = D_S / (S1 * f_1.min(f_2) + S2);


    let exp = (f_2 - f_1).abs() * s;

    // Find out why curve maximum doesn't reach 1.0 without fix
    a_1 * a_2 * (consts::E.powf(-A * exp) - consts::E.powf(-B * exp)) * 5.5f32
}

// Calculates the dissonance between two frequencies
// Currently doesn't differentiates between pitch of inverse ratios
pub fn estimate(f_heard: f32, i_heard:f32, f_inst: f32) -> f32 {
    let mut dis = 0f32;

    //dbg!(i_heard);

    if i_heard < 0.0000001f32 {
        return 0f32;
    }

    dis += dissonance(f_heard, i_heard, f_inst, 1f32);
    
    for i in 2 .. 200 {
        let harmonic_freq_high = f_inst * i as f32;
        let harmonic_freq_low = f_inst / i as f32;
        let harmonic_int = 1.3f32.powi(1 - i);
        let harmonic_dis =
            dissonance(f_heard, i_heard, harmonic_freq_high, harmonic_int)
            + dissonance(f_heard, i_heard, harmonic_freq_low, harmonic_int);
        dis += harmonic_dis;
    }

    //panic!();
    dis
}
