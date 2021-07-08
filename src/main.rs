use pulse::stream::Direction;
use pulse::sample::{Spec, Format};
use psimple::Simple;
use std::fs;
use std::io::prelude::*;
use rodio::{Sink, buffer::SamplesBuffer, OutputStream, source::Source};
use cpal::Sample;
use biquad::*;

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
    
    let cutoff_freq = (200.0).hz();
    let sampling_freq = (44100.0).hz();

    let coeffs = Coefficients::<f32>::from_params(Type::LowPass, sampling_freq, cutoff_freq, Q_BUTTERWORTH_F32).unwrap();

    let mut biquad = DirectForm1::<f32>::new(coeffs);
    
    while true {
        
        let mut buffer1: [u8; 4] = [0; 4]; // length has to be a multiple of 4
        s1.read(&mut buffer1).unwrap();

        let mut input_vec = Vec::new(); 
        for i in (0..buffer1.len()).step_by(2) {
            let two_bytes: [u8; 2] = [buffer1[i], buffer1[i+1]];
            let sample = i16::from_ne_bytes(two_bytes);
            let float_sample = sample.to_f32();
            input_vec.push(float_sample);
        }

        let mut output_vec = Vec::new();
        
        for elem in input_vec {
            output_vec.push(biquad.run(elem).to_i16());
            // output_vec.push(elem.to_u16());
        }
        
        let mut buffer2: [u8; 4] = [0; 4];
        for i in (0..output_vec.len()) {
            let two_bytes: [u8; 2] = i16::to_ne_bytes(output_vec[i]);
            buffer2[2*i] = two_bytes[0];
            buffer2[2*i + 1] = two_bytes[1];
        }
        
        s2.write(&buffer2[..]).unwrap();
    }
    
    
    
}

