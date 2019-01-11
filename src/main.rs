extern crate jack;
use std::io;
use std::result::Result;
use std::result::Result::Ok;

fn main() -> Result<(), jack::Error> {
    let (client, _status):(jack::Client, jack::ClientStatus) = jack::Client::new("rasta", jack::ClientOptions::NO_START_SERVER)?;

    // register ports
    let in_b = client
        .register_port("guitar_in", jack::AudioIn::default())?;
    let mut out_a = client
        .register_port("rasta_out_l", jack::AudioOut::default())?;
    let mut out_b = client
        .register_port("rasta_out_r", jack::AudioOut::default())?;

    let process_callback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let out_a_p = out_a.as_mut_slice(ps);
        let out_b_p = out_b.as_mut_slice(ps);
        let in_b_p = in_b.as_slice(ps);
        out_a_p.clone_from_slice(&in_b_p);
        out_b_p.clone_from_slice(&in_b_p);
        println!("writing {}", out_a_p.iter().fold(0f32, |a, b| a + b));
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
