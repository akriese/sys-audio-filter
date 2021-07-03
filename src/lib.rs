pub mod implementations {
    extern crate cpal;
    extern crate anyhow;
    use cpal::{SampleFormat};
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use rodio::{Sample, Sink, OutputStream, buffer::SamplesBuffer};

    #[derive(Copy, Clone)]
    pub struct StreamConfig {
        pub sample_rate: u32,
        pub channels: u16,
    }

    pub trait FilterBox {
        fn init(&mut self) -> Result<(), anyhow::Error>;
        fn play(&self) -> Result<(), anyhow::Error>;
        fn set_filter(&self, freq: usize, target_high_pass: bool);
    }

    pub struct CpalMgr {
        input_device: cpal::Device,
        output_device: cpal::Device,
        //in_cfg: cpal::SupportedStreamConfig,
        //out_cfg: cpal::SupportedStreamConfig,
        //sink: Arc<rodio::Sink>,
        //stream_handle: rodio::OutputStreamHandle,
        //low_pass:
        //high_pass:
        is_finished: Arc<AtomicBool>,
    }

    fn put_to_sink<R>(data: &[R], sink: &Arc<Sink>, channels: u16, sample_rate: u32)
        where
            R: Sample + Send + 'static, // idk why, but this only works with the 'static
        {
            // create new source from data and stream configuration
            //let source = InputStreamWrapper::new(data.to_vec(), in_conf);

            // apparently, there already is an alternative for our Wrapper:
            let source = SamplesBuffer::new(channels, sample_rate, data);
            //Source::low_pass(source, 1000);
            sink.append(source);

            //This is an alternative, which has worse quality than using the sink
            //stream_handle.play_raw(source).expect("Error while playbacking!");
        }

    impl CpalMgr {
        pub fn new() -> Result<CpalMgr, anyhow::Error>{
            let host = cpal::default_host();
            let input_device = host.input_devices()?
                .find(|x| x.name().map(|y| y == "CABLE Output (VB-Audio Virtual Cable)")
                      .unwrap_or(false))
                .expect("Failed to find input device!");

            let output_device = host.output_devices()?
                .find(|x| x.name().map(|y| y == "Lautsprecher (Realtek(R) Audio)")
                      .unwrap_or(false))
                .expect("Failed to find input device!");


            let is_finished = Arc::new(AtomicBool::new(false));


            Result::Ok(CpalMgr{ input_device, output_device, is_finished })
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
            let in_cfg = self.input_device.default_input_config()?;
            println!("Default input config: {:?}", in_cfg);

            let out_cfg = self.output_device.default_output_config()?;
            println!("Default output config: {:?}", out_cfg);

            let sink = Arc::new(Sink::try_new(&stream_handle).expect("couldnt build sink"));
            //let sink_clone = self.sink.clone();
            let sink_clone = sink.clone();

            let stream_conf = StreamConfig{
                sample_rate: in_cfg.sample_rate().0,
                channels: in_cfg.channels()
            };

            let err_fn = |err| eprintln!("an error occurred on either audio stream: {}", err);


            let in_stream =
                match in_cfg.sample_format() {
                    SampleFormat::F32 =>
                        self.input_device.build_input_stream(&in_cfg.clone().into(),
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            let source = SamplesBuffer::new(stream_conf.channels, stream_conf.sample_rate, data);
                            sink_clone.append(source);
                            //put_to_sink::<f32>(data, &sink_clone, in_conf.channels, in_conf.sample_rate);
                        }, err_fn),
                    SampleFormat::I16 =>
                        self.input_device.build_input_stream(&in_cfg.clone().into(),
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            put_to_sink::<i16>(data, &sink_clone, stream_conf.channels, stream_conf.sample_rate);
                        }, err_fn),
                    SampleFormat::U16 =>
                        self.input_device.build_input_stream(&in_cfg.clone().into(),
                        move |data: &[u16], _: &cpal::InputCallbackInfo| {
                            put_to_sink::<u16>(data, &sink_clone, stream_conf.channels, stream_conf.sample_rate);
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

        fn set_filter(&self, freq: usize, target_high_pass: bool) {

        }
    }
}
