pub mod implementations {
    extern crate cpal;
    extern crate anyhow;
    use std::io::{Write, stdin, stdout};
    use cpal::{SampleFormat};
    use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use rodio::{Sample, Sink, OutputStream, buffer::SamplesBuffer};
    use biquad::{Biquad, frequency::{ToHertz}, Coefficients, DirectForm1, Type::{LowPass, HighPass}};
    use biquad::Q_BUTTERWORTH_F32;

    pub trait FilterBox {
        fn init(&mut self) -> Result<(), anyhow::Error>;
        fn play(&self) -> Result<(), anyhow::Error>;
        fn set_filter(&self, freq: f32, target_high_pass: bool);
        fn is_finished(&self) -> bool;
        fn finish(&self);
    }

    pub struct CpalMgr {
        input_device: cpal::Device,
        output_device: cpal::Device,
        in_cfg: cpal::SupportedStreamConfig,
        out_cfg: cpal::SupportedStreamConfig,
        sample_rate: u32,
        channels: u16,
        low_pass: Arc<Mutex<DirectForm1<f32>>>,
        high_pass: Arc<Mutex<DirectForm1<f32>>>,
        is_finished: Arc<AtomicBool>,
    }

    fn put_to_sink<R>(data: &[R], sink: &Arc<Sink>, channels: u16, sample_rate: u32)
        where
            R: Sample + Send + 'static, // idk why, but this only works with the 'static
        {
            let source = SamplesBuffer::new(channels, sample_rate, data);
            sink.append(source);
        }

    impl CpalMgr {
        pub fn new() -> Result<CpalMgr, anyhow::Error>{
            let host = cpal::default_host();
            let (input_device, output_device) = CpalMgr::choose_input_output(&host).unwrap();

            let in_cfg = input_device.default_input_config()?;
            println!("Default input config: {:?}", in_cfg);

            let out_cfg = output_device.default_output_config()?;
            println!("Default output config: {:?}", out_cfg);


            let fs = in_cfg.sample_rate().0;
            let coeffs = Coefficients::<f32>::from_params(LowPass, fs.hz(), 20000.hz(), Q_BUTTERWORTH_F32).unwrap();
            let is_finished = Arc::new(AtomicBool::new(false));
            let channels = in_cfg.channels();

            Result::Ok(
                CpalMgr{ input_device, output_device, in_cfg, sample_rate: fs, channels, low_pass: Arc::new(Mutex::new(DirectForm1::<f32>::new(coeffs))), high_pass: Arc::new(Mutex::new(DirectForm1::<f32>::new(coeffs))), is_finished })
        }

        fn choose_input_output(host: &cpal::Host) -> Result<(cpal::Device, cpal::Device), anyhow::Error> {
            let input_device = CpalMgr::choose_device(host, true)?;
            let output_device = CpalMgr::choose_device(host, false)?;

            Result::Ok((input_device, output_device))
        }

        fn choose_device(host: &cpal::Host, target_input: bool) -> Result<cpal::Device, anyhow::Error> {
            let mut input = String::new();

            let mut devices = if target_input {
                println!("Input devices:");
                host.input_devices()?
            }
            else {
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
                }
                else {
                    host.default_output_device()
                }
            }
            else {
                devices = if target_input {
                    host.input_devices()?
                }
                else {
                    host.output_devices()?
                };
                devices.nth(index as usize)
            }.unwrap();

            println!("You have chosen {:?} as device!\n", device.name());

            Result::Ok(device)
        }
    }

    impl FilterBox for CpalMgr {
        fn init(&mut self) -> Result<(), anyhow::Error> {
            let host = cpal::default_host();
            self.input_device = host.input_devices()?
                .find(|x| x.name().map(|y| y == "CABLE Output (VB-Audio Virtual Cable)")
                      .unwrap_or(false))
                .expect("Failed to find input device!");

            self.output_device = host.output_devices()?
                .find(|x| x.name().map(|y| y == "Lautsprecher (Realtek(R) Audio)")
                      .unwrap_or(false))
                .expect("Failed to find input device!");

            self.is_finished = Arc::new(AtomicBool::new(false));

            Result::Ok(())
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

            let in_stream =
                match self.in_cfg.sample_format() {
                    SampleFormat::F32 =>
                        self.input_device.build_input_stream(&self.in_cfg.clone().into(),
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            let filtered_data: Vec<f32> = data.iter().map(|x| low_pass_cpy.lock().unwrap().run(*x)).collect();
                            let filtered_data: Vec<f32> = filtered_data.iter().map(|x| high_pass_cpy.lock().unwrap().run(*x)).collect();
                            let source = SamplesBuffer::new(channels_cpy, sample_rate_cpy, filtered_data);
                            sink_clone.append(source);
                            //put_to_sink::<f32>(data, &sink_clone, in_conf.channels, in_conf.sample_rate);
                        }, err_fn),
                    SampleFormat::I16 =>
                        self.input_device.build_input_stream(&self.in_cfg.clone().into(),
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            put_to_sink::<i16>(data, &sink_clone, channels_cpy, sample_rate_cpy);
                        }, err_fn),
                    SampleFormat::U16 =>
                        self.input_device.build_input_stream(&self.in_cfg.clone().into(),
                        move |data: &[u16], _: &cpal::InputCallbackInfo| {
                            put_to_sink::<u16>(data, &sink_clone, channels_cpy, sample_rate_cpy);
                        }, err_fn),
                }
            .unwrap();

            // use Ctrl+C handler to interrupt infinite sleeping loop
            let is_finished_cln = self.is_finished.clone();
            ctrlc::set_handler(move || {
                is_finished_cln.store(true, Ordering::Relaxed);
                println!("Keyboard Interrupt received!");
            }).expect("Error setting Ctrl+C handler");

            // start playback
            in_stream.play()?;
            //self.sink.sleep_until_end();
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
                self.high_pass.lock().unwrap().update_coefficients(Coefficients::<f32>::from_params(HighPass, self.sample_rate.hz(), freq.hz(), Q_BUTTERWORTH_F32).unwrap());
            }
            else {
                self.low_pass.lock().unwrap().update_coefficients(Coefficients::<f32>::from_params(LowPass, self.sample_rate.hz(), freq.hz(), Q_BUTTERWORTH_F32).unwrap());
            }
        }

        fn is_finished(&self) -> bool {
            self.is_finished.load(Ordering::Relaxed)
        }

        fn finish(&self) {
            self.is_finished.store(true, Ordering::Relaxed);
        }
    }
}
