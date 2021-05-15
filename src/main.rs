use std::fs::File;
use std::io::BufReader;
use std::string::String;
use std::time::Duration;
use rodio::{Decoder, OutputStream, Sink, source::{Source, SineWave}};
use std::env;


fn play_sine(frequency: u32, seconds: f32) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let source = SineWave::new(frequency).take_duration(Duration::from_secs_f32(seconds));
    sink.append(source);
    sink.sleep_until_end();
}

fn play_mp3(filename: &String) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let file = BufReader::new(File::open(filename).unwrap());
    let source = Decoder::new(file).unwrap();
    sink.append(source);
    sink.sleep_until_end();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = &args[1];
    match mode.as_ref() {
        "sine" => {
            let freq: u32 = args[2].trim().parse().expect("Frequency not given as a number!");
            let dur: f32 = args[3].trim().parse().expect("Duration not given as a number!");
            play_sine(freq, dur);
            println!("Sine wave of frequency {} played for {} seconds!", freq, dur);
        },
        "mp3" => {
            let sound_file_name = &args[2];
            play_mp3(sound_file_name);
            println!("The song is over!");
        },
        _ => println!("Unknown mode!"),
    }
}
