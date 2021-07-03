use pulse::stream::Direction;
use pulse::sample::{Spec, Format};
use psimple::Simple;
use std::fs;
use std::io::prelude::*;

use rodio::{Sink, buffer::SamplesBuffer, OutputStream, source::Source};

fn main() {
    // To do: Verallgemeinern
    let spec = Spec {
        format: Format::S16NE,
        channels: 2,
        rate: 44100,
    };

    assert!(spec.is_valid());

    // To do: User gibt Source und Sink an oder wählt aus Liste aus
    let input_device = "test.monitor";

    let s1 = Simple::new(
        None,                // Use the default server
        "FooApp",            // Our application’s name
        Direction::Record, // We want a stream for recording
        Some(&input_device),           
        "Music",             // Description of our stream
        &spec,               // Our sample format
        None,                // Use default channel map
        None                 // Use default buffering attributes
    ).unwrap();

    let output_device = "alsa_output.pci-0000_00_1b.0.analog-stereo";

    let s2 = Simple::new(
        None,                // Use the default server
        "FooApp",            // Our application’s name
        Direction::Playback, // We want a playback stream
        Some(&output_device),            
        "Music",             // Description of our stream
        &spec,               // Our sample format
        None,                // Use default channel map
        None                 // Use default buffering attributes
    ).unwrap();

    
    while true {
        let mut buffer: [u8; 4] = [0; 4];
        s1.read(&mut buffer).unwrap();

        s2.write(&buffer[..]).unwrap();
    }
    
    
    
}

