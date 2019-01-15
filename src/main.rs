#![feature(drain_filter)]
extern crate cpal;
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
use clampf::clamp;
use crossterm::style;
use crossterm::Color;

use cpal::StreamData::Input;
use cpal::UnknownTypeInputBuffer::{U16, I16, F32};
use cpal::Sample;

//setup ports, and start threads
fn main() {
    // Setup the default input device and stream with the default input format.
    let device = cpal::default_input_device()
        .expect("Failed to get default input device");
    println!("Default input device: {}", device.name());

    // Get the default sound input format
    let format = device.default_input_format()
        .expect("Failed to get default input format");
    println!("Default input format: {:?}", format);

    // Start the cpal input stream
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_input_stream(&device, &format)
        .expect("Failed to build input stream");
    event_loop.play_stream(stream_id);

    // Prepare the thread communication tools
    let recording = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let recording_2 = recording.clone();
    let (sender, receiver) = channel::<Vec<f32>>();

    // Await user input
    println!("Press enter/return to start reading frequencies ...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    // Spawn the audio input reading thread
    std::thread::spawn(move || {
        event_loop.run(move |_, data| {
            // If we're done recording, return early.
            if !recording_2.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }

            // Otherwise send input data to fourier thread
            if let Input{buffer: input_data} = data
            {
                let float_buffer = match input_data {
                    U16(buffer) => {
                        buffer.iter().map(|u| u.to_f32()).collect_vec()
                    },
                    I16(buffer) => {
                        buffer.iter().map(|i| i.to_f32()).collect_vec()
                    },
                    F32(buffer) => {
                        buffer.to_vec()
                    }
                };
                sender.send(float_buffer).unwrap();
            }
        });
    });

    thread::spawn(move || {fourier_thread(receiver)});

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();
}

//receives audio input, start FFT on most recent data and display results
fn fourier_thread(receiver:Receiver<Vec<f32>>)
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
        if queue.len() < 16
        {
            queue.push_front(receiver.recv().unwrap());
            continue;
        }
        //if too much input was aggregated, get rid of the oldest
        queue.truncate(16);
        //concatenate audio buffers, in order
        let mut vec:Vec<f32> = vec!();
        for input in queue.iter().rev()
        {
            vec.extend(input);
        }
        //apply fft and extract frequencies
        let vec = fourier_analysis(&vec[..], &mut planner, None);// Some(&mask));
        //calculate and display results
        calculate_scores(vec);
    }
}

fn fourier_analysis(
    vec:&[f32],
    planner:&mut FFTplanner<f32>,
    mask:Option<&Vec<f32>>) -> Vec<Complex<f32>>
{
    //setup fft parameters
    let len = vec.len();
    let mut fft_in = vec.iter().map(|&f| Complex{re:f, im:0f32}).collect_vec();
    let mut fft_out = vec![Complex::default(); len];
    let fft = planner.plan_fft(len);
    
    //process fft
    fft.process(&mut fft_in, &mut fft_out);

    //discard useless data
    fft_out.truncate(len / 2);
    fft_out.remove(0);
    //map results to frequencies and intensity
    for (Complex{re:a, im:b}, i) in fft_out.iter_mut().zip(1..)
    {
        //calculate intensity
        *b = (*a * *a + *b * *b).sqrt();
        //calculate frequency
        *a = i as f32 * 48000f32 / len as f32;
        //noise masking, currently unused
        if let Some(vec) = mask {
            if *b > vec[i] {
                *b -= vec[i];
            } else {
                *b = 0f32;
            }
        }
        //reducing intensity of frequencies out of human hearing range
        *b *= a_weigh_frequency(*a);
    }

    //sort by intensity
    fft_out.sort_by(|b, a| a.im.partial_cmp(&b.im).unwrap());

    // print 9 most intense frequencies (you'll never believe number 4)
    // for &Complex{re:a, im:b} in fft_out.iter().take(9)
    // {
    //     print!("{:^5.0}:{:^6.2} ", a, b);
    // }
    // println!("");

    fft_out
}

//https://fr.mathworks.com/matlabcentral/fileexchange/46819-a-weighting-filter-with-matlab
//reduce frequency intensity based on human perception
fn a_weigh_frequency(freq:f32) -> f32
{
    let c1 = 12194.217f32.powi(2);
    let c2 = 20.598997f32.powi(2);
    let c3 = 107.65265f32.powi(2);
    let c4 = 737.86223f32.powi(2);
    //evaluate the A-weighting filter in the frequency domain
    let freq = freq.powi(2);
    let num = c1*(freq.powi(2));
    let den = (freq+c2) * ((freq+c3)*(freq+c4)).sqrt() * (freq+c1);
    1.2589f32 * num / den
}

const NOTE_NAMES:[&str; 12] = [" C ", " C#", " D ", " D#", " E ", " F ", " F#", " G ", " G#", " A ", " A#", " B "];
const NOTE_NAMES_FRENCH:[&str; 12] = ["Do ", "Do#", "Ré ", "Ré#", "Mi ", "Fa ", "Fa#", "So ", "So#", "La ", "La#", "Si "];
const MAX_NOTE:usize = 88;
const BASE_NOTE:usize = 12 * 4 + 10; // C1 + 4 octaves + 10 == A4
const BASE_FREQUENCY:f32 = 440f32; //frequency of A4


fn calculate_scores(frequencies:Vec<Complex<f32>>)
{
    let mut scores:Vec<f32> = Vec::with_capacity(37);
    let mut min = std::f32::INFINITY;
    let mut max = std::f32::NEG_INFINITY;

    //for every note, calculate dissonance score
    for i in 0 ..= MAX_NOTE {
        let mut score = 0f32;
        let diff_a:i32 = i as i32 - BASE_NOTE as i32; 
        let hz = BASE_FREQUENCY * 2f32.powf(diff_a as f32 / 12f32);
        //for Complex{re:a, im:b} in [220, 440, 880].iter().map(|&hz| Complex{re:hz as f32, im:100f32})
        for &Complex{re:a, im:b} in frequencies.iter()
        {
            score += dissonance(a, hz) * b;
        }
        min = min.min(score);
        max = max.max(score);
        scores.push(score);
    }
    //normalize score
    for score in scores.iter_mut()
    {
        *score = (*score - min) / (max - min);
        *score = score.powf(0.5f32);
    }
    //display fretboard
    display_guitar(&scores[..]);
}

const GUITAR_STRING_LENGTH:usize = 44;
const GUITAR_STRINGS:[usize; 6] = [16 + 0, 16 + 5, 16 + 10, 16 + 15, 16 + 19, 16 + 24];

fn display_guitar(scores:&[f32])
{
    println!("");
    for &j in GUITAR_STRINGS.iter().rev()
    {
        for i in j .. j + GUITAR_STRING_LENGTH
        {
            let name = NOTE_NAMES[i % 12];
            let score = scores[i];
            let gradient = (clamp(score) * 255f32) as u8;
            let style = style(name)
                .with(Color::DarkBlue)
                .on(Color::Rgb{r:gradient, g:255 - gradient, b:0});
            print!("{}", style);
        }
        println!("");
    }
}

const DISSONANCE_OCTAVE_RATIO:f32 = 1.02;

fn dissonance(f_1:f32, f_2:f32) -> f32
{
    //find the ratio between the two frequencies
    let ratio = if f_1 < f_2
        { f_2 / f_1 }
        else 
        { f_1 / f_2 };

    //get number of full octaves
    let octaves = ratio.log2() as i32;
    //reduce to less than 1 octave
    let ratio = ratio / 2f32.powi(octaves);
    //increase dissonance with octave distance
    let cons = DISSONANCE_OCTAVE_RATIO.powi(octaves);

    cons * DISSONANCE_TABLE[((ratio - 1f32) * 200f32) as usize].powf(2f32)
}

const DISSONANCE_TABLE:[f32;200] = [
    0.4040681240751095f32,
    0.518491547915879f32,
    0.6611341146326413f32,
    0.7631741820912841f32,
    0.8326376742616903f32,
    0.8862658468103187f32,
    0.9309477002273211f32,
    0.9711205625170499f32,
    0.9946160947208659f32,
    0.991230802407768f32,
    0.9809293321883689f32,
    0.9707669063146271f32,
    0.9624920397833486f32,
    0.959149103393275f32,
    0.9608761188050832f32,
    0.9015853645185463f32,
    0.8259992785809807f32,
    0.8065139738203031f32,
    0.7879984516702505f32,
    0.7665693511115361f32,
    0.7473925248093638f32,
    0.7280599570482603f32,
    0.7066049145885793f32,
    0.6891217057800691f32,
    0.6675042760464811f32,
    0.6455684584708721f32,
    0.6355998969776212f32,
    0.6323703828770594f32,
    0.6197383648306927f32,
    0.6184879774847035f32,
    0.6272555650321663f32,
    0.625708519355333f32,
    0.6168315152743818f32,
    0.601608484275341f32,
    0.6044767345471987f32,
    0.614952278585836f32,
    0.6126434589414349f32,
    0.6111331865892102f32,
    0.6042446076635485f32,
    0.5840872281793146f32,
    0.5669474630447554f32,
    0.5777632233175055f32,
    0.5882538210552712f32,
    0.5857627319170475f32,
    0.5774193687476719f32,
    0.5732736002506447f32,
    0.5684455059166874f32,
    0.5593734107118399f32,
    0.5403863204378745f32,
    0.5119911545250232f32,
    0.48960695286362477f32,
    0.5010492558066666f32,
    0.5186110765346008f32,
    0.5278462536712812f32,
    0.5314059761644541f32,
    0.5314680118866273f32,
    0.5283213489090068f32,
    0.5199940764355808f32,
    0.5277553174017624f32,
    0.5337315020127792f32,
    0.530020951924944f32,
    0.5335031099090763f32,
    0.5309620519855105f32,
    0.5228517879683143f32,
    0.5072620721565778f32,
    0.4802291849051312f32,
    0.4509629658022273f32,
    0.4476263481475049f32,
    0.47502966572379357f32,
    0.5059009031498742f32,
    0.5233559706044802f32,
    0.5302278581819865f32,
    0.53344074073124f32,
    0.5328427766812013f32,
    0.5317017516088602f32,
    0.525796989209394f32,
    0.5307748946533593f32,
    0.5324185169040816f32,
    0.528166911134973f32,
    0.5158548805935886f32,
    0.5040989049586294f32,
    0.5149374453600305f32,
    0.524557552368055f32,
    0.5272060037478273f32,
    0.5285672579970235f32,
    0.5244931869225034f32,
    0.5219724240669379f32,
    0.5291211262802303f32,
    0.5313258450225792f32,
    0.5293548462814028f32,
    0.533501568885275f32,
    0.5334266199927657f32,
    0.5345638532896114f32,
    0.5310203954677162f32,
    0.524039588591444f32,
    0.5157775051534693f32,
    0.5009050384835702f32,
    0.4729815943361275f32,
    0.4367523378400455f32,
    0.3983024223752327f32,
    0.3762356915632271f32,
    0.39533859007991523f32,
    0.43000108627399913f32,
    0.46372032768518623f32,
    0.48796017620539267f32,
    0.5004436329836792f32,
    0.5048859984681525f32,
    0.509548373733774f32,
    0.5086643335391385f32,
    0.5034691395312187f32,
    0.4994563628155406f32,
    0.49446601001143586f32,
    0.49260719322586743f32,
    0.48748854080670934f32,
    0.4783904465715634f32,
    0.4764717315067487f32,
    0.477419835630734f32,
    0.4734867305861974f32,
    0.4695615339625768f32,
    0.46041764615635195f32,
    0.45172854539113116f32,
    0.46003113502038806f32,
    0.468820781611586f32,
    0.47060088972470715f32,
    0.4685358431512413f32,
    0.46433940858925693f32,
    0.46844183623987223f32,
    0.4677522981937269f32,
    0.46879903117209776f32,
    0.46492143711127165f32,
    0.4588674511452214f32,
    0.4460365231679675f32,
    0.42534091590896395f32,
    0.40564435163273604f32,
    0.4072948529381466f32,
    0.4267633520489287f32,
    0.4420369152288067f32,
    0.4484058493470655f32,
    0.45058506831182543f32,
    0.44904002017272626f32,
    0.4457339835849845f32,
    0.44720149365359585f32,
    0.443661778893495f32,
    0.4390728332729579f32,
    0.442635663447821f32,
    0.44316364742944636f32,
    0.4422727228723008f32,
    0.43923619687055426f32,
    0.4324380657462832f32,
    0.4192019736080588f32,
    0.40888485338583225f32,
    0.41548794217677576f32,
    0.42617096347443606f32,
    0.43015471573392117f32,
    0.4306409924873834f32,
    0.4328010320423884f32,
    0.43625588764439427f32,
    0.44031574228662806f32,
    0.44127240106502674f32,
    0.43731102870846483f32,
    0.4314001025862595f32,
    0.44043387375510534f32,
    0.4496577524191028f32,
    0.45452168687614003f32,
    0.45703547225588803f32,
    0.45810222273701406f32,
    0.45443954786195917f32,
    0.45545366690044087f32,
    0.46248284308178866f32,
    0.467509966550683f32,
    0.46956675306436424f32,
    0.46670579488612807f32,
    0.47024190945861344f32,
    0.47527018428101386f32,
    0.4768573881572975f32,
    0.4762277556613538f32,
    0.48060764796549327f32,
    0.4833355572843635f32,
    0.48694196177840676f32,
    0.4951033039506827f32,
    0.5022336848153964f32,
    0.5093275299405057f32,
    0.5164286934548593f32,
    0.5236074177989061f32,
    0.5308239426909694f32,
    0.5369984287421588f32,
    0.5347254770838136f32,
    0.5307509730087f32,
    0.5266054572500639f32,
    0.52164212587409f32,
    0.5176603762450267f32,
    0.5135321584936813f32,
    0.5041333716324375f32,
    0.4840657788000614f32,
    0.4546515552801309f32,
    0.42531160324380457f32,
    0.39453650432766396f32,
    0.3618171142230009f32,
    0.3256405570375345f32,
    0.2819836211528104f32,
    ];
