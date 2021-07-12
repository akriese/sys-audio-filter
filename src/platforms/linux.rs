use anyhow;
use biquad::Q_BUTTERWORTH_F32;
use biquad::{
    frequency::ToHertz,
    Biquad, Coefficients, DirectForm1,
    Type::{HighPass, LowPass},
};
use cpal::Sample;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use psimple::Simple;
use pulse::sample::{Format, Spec};
use pulse::stream::Direction;
use std::process::Command;

use crate::platforms::FilterBox;

pub struct PaMgr {
    spec: Spec,
    source: Simple,
    sink: Simple,
    output_device: String,
    pub sample_rate: f32,
    low_pass: Arc<Mutex<DirectForm1<f32>>>,
    high_pass: Arc<Mutex<DirectForm1<f32>>>,
    is_finished: Arc<AtomicBool>,
}

impl PaMgr {
    pub fn new() -> Result<PaMgr, anyhow::Error> {
        let _output = Command::new("pactl")
            .arg("load-module")
            .arg("module-null-sink")
            .arg("sink_name=test")
            .output()
            .expect("Failed to execute command");

        let is_finished = Arc::new(AtomicBool::new(false));

        let spec = Spec {
            format: Format::S16NE,
            channels: 2,
            rate: 44100,
        };

        assert!(spec.is_valid());

        let input_device = "test.monitor";

        let source = Simple::new(
            None,              // Use the default server
            "FooApp",          // Our application’s name
            Direction::Record, // We want a stream for recording
            Some(&input_device),
            "Music", // Description of our stream
            &spec,   // Our sample format
            None,    // Use default channel map
            None,    // Use default buffering attributes
        )
        .unwrap();

        let output_device = "alsa_output.pci-0000_00_1b.0.analog-stereo".to_string();

        let sink = Simple::new(
            None,                // Use the default server
            "FooApp",            // Our application’s name
            Direction::Playback, // We want a playback stream
            //Some(&output_device),
            None,
            "Music", // Description of our stream
            &spec,   // Our sample format
            None,    // Use default channel map
            None,    // Use default buffering attributes
        )
        .unwrap();

        let sample_rate = spec.rate as f32;
        let cutoff_freq1 = sample_rate / 2f32;
        let cutoff_freq2 = 10.0;

        let coeffs1 = Coefficients::<f32>::from_params(
            LowPass,
            sample_rate.hz(),
            cutoff_freq1.hz(),
            Q_BUTTERWORTH_F32,
        )
        .unwrap();
        let coeffs2 = Coefficients::<f32>::from_params(
            HighPass,
            sample_rate.hz(),
            cutoff_freq2.hz(),
            Q_BUTTERWORTH_F32,
        )
        .unwrap();

        let low_pass = Arc::new(Mutex::new(DirectForm1::<f32>::new(coeffs1)));
        let high_pass = Arc::new(Mutex::new(DirectForm1::<f32>::new(coeffs2)));

        Result::Ok(PaMgr {
            spec,
            source,
            sink,
            output_device,
            sample_rate,
            low_pass,
            high_pass,
            is_finished,
        })
    }
}

impl Drop for PaMgr {
    fn drop(&mut self) {
        let _output = Command::new("pactl")
            .arg("unload-module")
            .arg("module-null-sink")
            .output()
            .expect("Failed to execute command");

        println!("Dropping");
    }
}

impl FilterBox for PaMgr {
    fn set_volume(&self, factor: u16) {
        let mut owned_string: String = factor.to_string();
        let borrowed_string: &str = "%";

        owned_string.push_str(borrowed_string);

        let _output = Command::new("pactl")
            .arg("set-sink-volume")
            .arg(&self.output_device)
            .arg(owned_string)
            .output()
            .expect("Failed to execute command");
    }

    fn play(&self) -> Result<(), anyhow::Error> {
        while !self.is_finished() {
            let mut buffer1: [u8; 32] = [0; 32]; // length has to be a multiple of 4
            self.source.read(&mut buffer1).unwrap();

            let mut input_vec = Vec::new();
            for i in (0..buffer1.len()).step_by(2) {
                let two_bytes: [u8; 2] = [buffer1[i], buffer1[i + 1]];
                let sample = i16::from_ne_bytes(two_bytes);
                let float_sample = sample.to_f32();
                input_vec.push(float_sample);
            }

            let mut output_vec = Vec::new();

            for elem in input_vec {
                output_vec.push(
                    self.low_pass
                        .lock()
                        .unwrap()
                        .run(self.high_pass.lock().unwrap().run(elem))
                        .to_i16(),
                );
            }

            let mut buffer2: [u8; 32] = [0; 32];
            for i in 0..output_vec.len() {
                let two_bytes: [u8; 2] = i16::to_ne_bytes(output_vec[i]);
                buffer2[2 * i] = two_bytes[0];
                buffer2[2 * i + 1] = two_bytes[1];
            }

            self.sink.write(&buffer2[..]).unwrap();
        }

        Ok(())
    }

    fn set_filter(&self, freq: f32, target_high_pass: bool) {
        let sampling_freq = self.spec.rate as f32;
        if target_high_pass {
            let coeffs = Coefficients::<f32>::from_params(
                HighPass,
                sampling_freq.hz(),
                freq.hz(),
                Q_BUTTERWORTH_F32,
            )
            .unwrap();
            self.high_pass.lock().unwrap().update_coefficients(coeffs);
        } else {
            let coeffs = Coefficients::<f32>::from_params(
                LowPass,
                sampling_freq.hz(),
                freq.hz(),
                Q_BUTTERWORTH_F32,
            )
            .unwrap();
            self.low_pass.lock().unwrap().update_coefficients(coeffs);
        }
    }

    fn is_finished(&self) -> bool {
        self.is_finished.load(Ordering::Relaxed)
    }

    fn finish(&self) {
        self.is_finished.store(true, Ordering::Relaxed);
    }
}
