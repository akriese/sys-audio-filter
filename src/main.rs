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

    let is_little_endian = spec.format.is_le(); 

    match is_little_endian {
        Some(i) => { 
            if i {
                println!("little endian");
            } else {
                println!("big endian");
            }
        }
        None => println!("No information about endianess!"),
    }

    // To do: User gibt Source und Sink an oder wählt aus Liste aus
    let input_device = "test.monitor";

    let s1 = Simple::new(
        None,                // Use the default server
        "FooApp",            // Our application’s name
        Direction::Record, // We want a playback stream
        Some(&input_device),           // Use the default device
        "Music",             // Description of our stream
        &spec,               // Our sample format
        None,                // Use default channel map
        None                 // Use default buffering attributes
    ).unwrap();

    // Idee: kleinen Vektor als Buffer für Samples bauen (je zwei u8s zu u16s paaren) und in
    // Schleife solange der immer wieder gefüllt wird an rodio Sink anhängen

    /* 
    let mut recording: [u8; 2] = [0; 2];

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    while true {
        let mut vec: Vec<u16> = Vec::new();


        // let mut i = 0;
        // while i < 100000 {
        //    i += 1;
            s.read(&mut recording).unwrap();
            let sample_for_sink = u16::from_ne_bytes(recording);
            vec.push(sample_for_sink);
        // }

        let source_for_sink = SamplesBuffer::new(2, 44100, vec);

        sink.append(source_for_sink);

        sink.sleep_until_end();
    }
    */ 

    // let output_device = 

    let s2 = Simple::new(
        None,                // Use the default server
        "FooApp",            // Our application’s name
        Direction::Playback, // We want a playback stream
        None,               // Use the default device
        "Music",             // Description of our stream
        &spec,               // Our sample format
        None,                // Use default channel map
        None                 // Use default buffering attributes
    ).unwrap();


    while true {
        let mut buffer: [u8; 10] = [0; 10];
        s1.read(&mut buffer).unwrap();

        s2.write(&buffer).unwrap();
        s2.drain().unwrap();
    }

    // s.write(&samples).unwrap();
    // s.drain().unwrap();
    
    // let mut recording: [u8; 5500000] = [0; 5500000];
    // s.read(&mut recording).unwrap();

    // let mut f = fs::File::create("testOutput.raw").unwrap();
    // f.write_all(&recording).unwrap();

}

