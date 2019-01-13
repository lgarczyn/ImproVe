#![feature(drain_filter)]
extern crate jack;
extern crate itertools;
use std::io;
use std::vec;
use std::result::Result;
use std::result::Result::Ok;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::thread;
use rustfft::FFTplanner;
use rustfft::num_complex::Complex;
use itertools::Itertools;



fn main() -> Result<(), jack::Error> {
    let (client, _status):(jack::Client, jack::ClientStatus) = jack::Client::new("rasta", jack::ClientOptions::NO_START_SERVER)?;

    // register ports
    let in_b = client
        .register_port("guitar_in", jack::AudioIn::default())?;

    let (sender, receiver) = channel::<Vec<f32>>();

    thread::spawn(move || {fourrier_thread(receiver)});

    //start the rasta callback on the default audio input
    let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let in_b_p = in_b.as_slice(ps);
        
        sender.send(in_b_p.to_vec()).unwrap();

        jack::Control::Continue
    };
    let process = jack::ClosureProcessHandler::new(process_callback);
    let active_client = client.activate_async((), process)?;

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate()?;

    Ok(())
}

fn fourrier_thread(receiver:Receiver<Vec<f32>>)
{
    let mut planner = FFTplanner::<f32>::new(false);

    let mut queue = std::collections::VecDeque::new();
    loop
    {
        //aggregate all pending input
        for input in receiver.try_iter()
        {
            queue.push_front(input);
        }
        //if not enough input was aggregated, wait and try again
        if queue.len() < 32
        {
            queue.push_front(receiver.recv().unwrap());
            continue;
        }
        //if too much input was aggregated, get rid of the oldest
        queue.truncate(32);
        //aggregate input, oldest first
        let mut vec:Vec<f32> = vec!();
        for input in queue.iter().rev()
        {
            vec.extend(input);
        }
        //apply fft and extract frequencies
        fourrier_analysis(&vec[..], &mut planner);
    }
}

fn fourrier_analysis(vec:&[f32], planner:&mut FFTplanner<f32>)
{
    //setup fft parameters
    let len = vec.len();
    let mut fft_in = vec.iter().map(|f| Complex{re:*f, im:0f32}).collect_vec();
    let mut fft_out = vec![Complex::default(); len];
    let fft = planner.plan_fft(len);
    
    //process fft
    fft.process(&mut fft_in, &mut fft_out);

    //discard useless data
    fft_out.truncate(len / 2);
    //map results to frequencies and intensity
    for (Complex{re:a, im:b}, i) in fft_out.iter_mut().zip(0..)
    {
        *b = (*a * *a + *b * *b).sqrt();
        *a = i as f32 * 44100f32 / len as f32;
        *b *= a_weigh_frequency(*a);
    }

    //sort by intensity
    fft_out.sort_by(|b, a| a.im.partial_cmp(&b.im).unwrap());

    //print 9 most intense frequencies
    for &Complex{re:a, im:b} in fft_out.iter().take(9)
    {
        print!("{:^5.0}:{:^6.2} ", a, b);
    }
    println!("");
}

// https://fr.mathworks.com/matlabcentral/fileexchange/46819-a-weighting-filter-with-matlab
// reduce frequency intensity based on human perception
fn a_weigh_frequency(freq:f32) -> f32
{
    let c1 = 12194.217f32.powi(2);
    let c2 = 20.598997f32.powi(2);
    let c3 = 107.65265f32.powi(2);
    let c4 = 737.86223f32.powi(2);
    // evaluate the A-weighting filter in the frequency domain
    let freq = freq.powi(2);
    let num = c1*(freq.powi(2));
    let den = (freq+c2) * ((freq+c3)*(freq+c4)).sqrt() * (freq+c1);
    1.2589f32 * num / den
}
