use pulse::stream::Direction;
use pulse::sample::{Spec, Format};
use psimple::Simple;
use std::fs;
use std::io::prelude::*;

fn main() {
    let spec = Spec {
        format: Format::S16NE,
        channels: 2,
        rate: 44100,
    };
    assert!(spec.is_valid());

    let s = Simple::new(
        None,                // Use the default server
        "FooApp",            // Our application’s name
        Direction::Playback, // We want a playback stream
        None,                // Use the default device
        "Music",             // Description of our stream
        &spec,               // Our sample format
        None,                // Use default channel map
        None                 // Use default buffering attributes
    ).unwrap();


    // Hier Song-Datei öffnen und in Array einlesen
    let samples = fs::read("testOutput.raw").unwrap();
    // println!("Es sind soviele u8s drin: {}", samples.len());

    s.write(&samples).unwrap();
    s.drain().unwrap();
    
    // let mut recording: [u8; 5500000] = [0; 5500000];
    // s.read(&mut recording).unwrap();

    // let mut f = fs::File::create("testOutput.raw").unwrap();
    // f.write_all(&recording).unwrap();

}

