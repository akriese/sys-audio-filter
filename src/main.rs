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
    let file = BufReader::new(File::open(filename).unwrap());
    let source = Decoder::new(file).unwrap();
    stream_handle.play_raw(source.convert_samples());
    std::thread::sleep(std::time::Duration::from_secs(5));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let sound_file_name = &args[1];
    play_mp3(sound_file_name);
    println!("The song is over!");
}
