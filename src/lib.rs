pub mod implementations {
    extern crate anyhow;
    extern crate cpal;
    use biquad::Q_BUTTERWORTH_F32;
    use biquad::{
        frequency::ToHertz,
        Biquad, Coefficients, DirectForm1,
        Type::{HighPass, LowPass},
    };
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{Sample, SampleFormat};
    use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
    use std::io::{stdin, stdout, Write};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    };

    use pulse::stream::Direction;
    use pulse::sample::{Spec, Format};
    use psimple::Simple;
    use std::process::Command;

    pub trait FilterBox {
        fn play(&self) -> Result<(), anyhow::Error>;
        fn set_filter(&self, freq: f32, target_high_pass: bool);
        fn is_finished(&self) -> bool;
        fn finish(&self);
        fn set_volume(&self, factor: u16);
    }

    fn apply_filter(data: &mut Vec<f32>, filter: &Arc<Mutex<DirectForm1<f32>>>) {
        for x in data.iter_mut() {
            *x = filter.lock().unwrap().run(*x);
        }
    }

    pub struct CpalMgr {
        input_device: cpal::Device,
        output_device: cpal::Device,
        in_cfg: cpal::SupportedStreamConfig,
        //out_cfg: cpal::SupportedStreamConfig,
        sample_rate: u32,
        channels: u16,
        low_pass: Arc<Mutex<DirectForm1<f32>>>,
        high_pass: Arc<Mutex<DirectForm1<f32>>>,
        is_finished: Arc<AtomicBool>,
    }

    impl CpalMgr {
        
        pub fn new() -> Result<CpalMgr, anyhow::Error> {
            let host = cpal::default_host();
            let (input_device, output_device) = CpalMgr::choose_input_output(&host).unwrap();

            let in_cfg = input_device.default_input_config()?;
            println!("Default input config: {:?}", in_cfg);

            let out_cfg = output_device.default_output_config()?;
            println!("Default output config: {:?}", out_cfg);

            let fs = in_cfg.sample_rate().0;
            let coeffs =
                Coefficients::<f32>::from_params(LowPass, fs.hz(), 20000.hz(), Q_BUTTERWORTH_F32)
                    .unwrap();
            let is_finished = Arc::new(AtomicBool::new(false));
            let channels = in_cfg.channels();

            Result::Ok(CpalMgr {
                input_device,
                output_device,
                in_cfg,
                sample_rate: fs,
                channels,
                low_pass: Arc::new(Mutex::new(DirectForm1::<f32>::new(coeffs))),
                high_pass: Arc::new(Mutex::new(DirectForm1::<f32>::new(coeffs))),
                is_finished,
            })
        }

        fn choose_input_output(
            host: &cpal::Host,
        ) -> Result<(cpal::Device, cpal::Device), anyhow::Error> {
            let input_device = CpalMgr::choose_device(host, true)?;
            let output_device = CpalMgr::choose_device(host, false)?;

            Result::Ok((input_device, output_device))
        }

        fn choose_device(
            host: &cpal::Host,
            target_input: bool,
        ) -> Result<cpal::Device, anyhow::Error> {
            let mut input = String::new();

            let mut devices = if target_input {
                println!("Input devices:");
                host.input_devices()?
            } else {
                println!("Output devices:");
                host.output_devices()?
            };
            let mut device_count = 0; // unfortunately, size_hint() is not helpful here
            for (device_index, device) in devices.enumerate() {
                println!("{}. \"{}\"", device_index, device.name()?);
                device_count += 1;
            }
            print!("Choose a device by its index: ");
            stdout().flush()?;
            stdin().read_line(&mut input).expect("Error reading input");
            let mut index = -1;
            if input.trim().len() > 0 {
                index = input.trim().to_string().parse::<i16>().unwrap();
            }

            let device = if index < 0 || index >= device_count {
                if target_input {
                    host.default_input_device()
                } else {
                    host.default_output_device()
                }
            } else {
                devices = if target_input {
                    host.input_devices()?
                } else {
                    host.output_devices()?
                };
                devices.nth(index as usize)
            }
            .unwrap();

            println!("You have chosen {:?} as device!\n", device.name());

            Result::Ok(device)
        }
    }

    impl FilterBox for CpalMgr {
        fn set_volume(&self, factor: u16) {
            println!("Yo {}", factor);
        }

        fn play(&self) -> Result<(), anyhow::Error> {
            let (_stream, stream_handle) = OutputStream::try_from_device(&self.output_device)?;

            let sink = Arc::new(Sink::try_new(&stream_handle).expect("couldnt build sink"));
            let sink_clone = sink.clone();
            let channels_cpy = self.channels.clone();
            let sample_rate_cpy = self.sample_rate.clone();
            let low_pass_cpy = self.low_pass.clone();
            let high_pass_cpy = self.high_pass.clone();

            let err_fn = |err| eprintln!("an error occurred on either audio stream: {}", err);

            let in_stream = match self.in_cfg.sample_format() {
                SampleFormat::F32 => self.input_device.build_input_stream(
                    &self.in_cfg.clone().into(),
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mut filtered_data = data.to_vec();
                        apply_filter(&mut filtered_data, &low_pass_cpy);
                        apply_filter(&mut filtered_data, &high_pass_cpy);
                        let source =
                            SamplesBuffer::new(channels_cpy, sample_rate_cpy, filtered_data);
                        sink_clone.append(source);
                    },
                    err_fn,
                ),
                SampleFormat::I16 => self.input_device.build_input_stream(
                    &self.in_cfg.clone().into(),
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let mut filtered_data: Vec<f32> =
                            data.iter().map(|x| (*x).to_f32()).collect();
                        apply_filter(&mut filtered_data, &low_pass_cpy);
                        apply_filter(&mut filtered_data, &high_pass_cpy);
                        let filtered_data: Vec<i16> =
                            filtered_data.iter().map(|x| (*x).to_i16()).collect();
                        let source =
                            SamplesBuffer::new(channels_cpy, sample_rate_cpy, filtered_data);
                        sink_clone.append(source);
                    },
                    err_fn,
                ),
                SampleFormat::U16 => self.input_device.build_input_stream(
                    &self.in_cfg.clone().into(),
                    move |data: &[u16], _: &cpal::InputCallbackInfo| {
                        let mut filtered_data: Vec<f32> =
                            data.iter().map(|x| (*x).to_f32()).collect();
                        apply_filter(&mut filtered_data, &low_pass_cpy);
                        apply_filter(&mut filtered_data, &high_pass_cpy);
                        let filtered_data: Vec<u16> =
                            filtered_data.iter().map(|x| (*x).to_u16()).collect();
                        let source =
                            SamplesBuffer::new(channels_cpy, sample_rate_cpy, filtered_data);
                        sink_clone.append(source);
                    },
                    err_fn,
                ),
            }
            .unwrap();

            // use Ctrl+C handler to interrupt infinite sleeping loop
            let is_finished_cln = self.is_finished.clone();
            ctrlc::set_handler(move || {
                is_finished_cln.store(true, Ordering::Relaxed);
                println!("Keyboard Interrupt received!");
            })
            .expect("Error setting Ctrl+C handler");

            // start playback
            in_stream.play()?;
            sink.sleep_until_end();
            loop {
                if self.is_finished.load(Ordering::Relaxed) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(200));
            }

            Result::Ok(())
        }

        fn set_filter(&self, freq: f32, target_high_pass: bool) {
            if target_high_pass {
                self.high_pass.lock().unwrap().update_coefficients(
                    Coefficients::<f32>::from_params(
                        HighPass,
                        self.sample_rate.hz(),
                        freq.hz(),
                        Q_BUTTERWORTH_F32,
                    )
                    .unwrap(),
                );
            } else {
                self.low_pass.lock().unwrap().update_coefficients(
                    Coefficients::<f32>::from_params(
                        LowPass,
                        self.sample_rate.hz(),
                        freq.hz(),
                        Q_BUTTERWORTH_F32,
                    )
                    .unwrap(),
                );
            }
        }

        fn is_finished(&self) -> bool {
            self.is_finished.load(Ordering::Relaxed)
        }

        fn finish(&self) {
            self.is_finished.store(true, Ordering::Relaxed);
        }
    }

    pub struct PaMgr {
        spec:  Spec,
        source: Simple,
        sink: Simple,
        output_device: String,
        low_pass: Arc<Mutex<DirectForm1<f32>>>, 
        high_pass: Arc<Mutex<DirectForm1<f32>>>,
        is_finished: Arc<AtomicBool>,
    }

    impl PaMgr {
       pub fn new() -> Result <PaMgr, anyhow::Error> {
           
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
                None,                // Use the default server
                "FooApp",            // Our application’s name
                Direction::Record, // We want a stream for recording
                Some(&input_device),
                "Music",             // Description of our stream
                &spec,               // Our sample format
                None,                // Use default channel map
                None                 // Use default buffering attributes
            ).unwrap();
            
            let output_device = "alsa_output.pci-0000_00_1b.0.analog-stereo".to_string();

            let sink = Simple::new(
                None,                // Use the default server
                "FooApp",            // Our application’s name
                Direction::Playback, // We want a playback stream
                Some(&output_device),            
                "Music",             // Description of our stream
                &spec,               // Our sample format
                None,                // Use default channel map
                None                 // Use default buffering attributes
            ).unwrap();

            let cutoff_freq1 = 20000.0;
            let cutoff_freq2 = 10.0;
            let sampling_freq = spec.rate as f32;

            let coeffs1 = Coefficients::<f32>::from_params(LowPass, sampling_freq.hz(), cutoff_freq1.hz(), Q_BUTTERWORTH_F32).unwrap();

            let coeffs2 = Coefficients::<f32>::from_params(HighPass, sampling_freq.hz(), cutoff_freq2.hz(), Q_BUTTERWORTH_F32).unwrap();

            let low_pass = Arc::new(Mutex::new(DirectForm1::<f32>::new(coeffs1)));
            let high_pass = Arc::new(Mutex::new(DirectForm1::<f32>::new(coeffs2)));

            Result::Ok(PaMgr {
                spec,
                source,
                sink,
                output_device,
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
                let mut buffer1: [u8; 4] = [0; 4]; // length has to be a multiple of 4
                self.source.read(&mut buffer1).unwrap();

                let mut input_vec = Vec::new(); 
                for i in (0..buffer1.len()).step_by(2) {
                    let two_bytes: [u8; 2] = [buffer1[i], buffer1[i+1]];
                    let sample = i16::from_ne_bytes(two_bytes);
                    let float_sample = sample.to_f32();
                    input_vec.push(float_sample);
                }

                let mut output_vec = Vec::new();
                
                for elem in input_vec {
                    output_vec.push(self.low_pass.lock().unwrap().run(self.high_pass.lock().unwrap().run(elem)).to_i16());
                }
                
                let mut buffer2: [u8; 4] = [0; 4];
                for i in 0..output_vec.len() {
                    let two_bytes: [u8; 2] = i16::to_ne_bytes(output_vec[i]);
                    buffer2[2*i] = two_bytes[0];
                    buffer2[2*i + 1] = two_bytes[1];
                }
                
                self.sink.write(&buffer2[..]).unwrap();
            } 

            Ok(())
            
        }

        fn set_filter(&self, freq: f32, target_high_pass: bool) {
            let sampling_freq = self.spec.rate as f32;
            if target_high_pass {
                let coeffs = Coefficients::<f32>::from_params(HighPass, sampling_freq.hz(), freq.hz(), Q_BUTTERWORTH_F32).unwrap();
                self.high_pass.lock().unwrap().update_coefficients(coeffs);
            } else {
                let coeffs = Coefficients::<f32>::from_params(LowPass, sampling_freq.hz(), freq.hz(), Q_BUTTERWORTH_F32).unwrap();
                self.low_pass.lock().unwrap().update_coefficients(coeffs);
            }
        }

        fn is_finished(&self) -> bool {
            self.is_finished.load(Ordering::Relaxed)
        }

        fn finish(&self) {
            self.is_finished.store(true,Ordering::Relaxed);  
        }

    }

}
