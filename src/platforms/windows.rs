use crate::platforms::{FilterBox, get_max_freq};
use anyhow;
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

pub struct CpalMgr {
    input_device: cpal::Device,
    output_device: cpal::Device,
    in_cfg: cpal::SupportedStreamConfig,
    //out_cfg: cpal::SupportedStreamConfig,
    pub sample_rate: f32,
    channels: u16,
    low_pass: Arc<Mutex<DirectForm1<f32>>>,
    high_pass: Arc<Mutex<DirectForm1<f32>>>,
    is_finished: Arc<AtomicBool>,
}

fn apply_filter(data: &mut Vec<f32>, filter: &Arc<Mutex<DirectForm1<f32>>>) {
    for x in data.iter_mut() {
        *x = filter.lock().unwrap().run(*x);
    }
}

impl CpalMgr {
    pub fn new() -> Result<CpalMgr, anyhow::Error> {
        let host = cpal::default_host();
        let (input_device, output_device) = CpalMgr::choose_input_output(&host).unwrap();

        let in_cfg = input_device.default_input_config()?;
        println!("Default input config: {:?}", in_cfg);

        let out_cfg = output_device.default_output_config()?;
        println!("Default output config: {:?}", out_cfg);

        let fs = in_cfg.sample_rate().0 as f32;
        let coeffs = Coefficients::<f32>::from_params(
            LowPass,
            fs.hz(),
            get_max_freq(fs).hz(),
            Q_BUTTERWORTH_F32,
        )
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

    fn choose_device(host: &cpal::Host, target_input: bool) -> Result<cpal::Device, anyhow::Error> {
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
        let sample_rate_cpy = self.sample_rate.clone() as u32;
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
                    let source = SamplesBuffer::new(channels_cpy, sample_rate_cpy, filtered_data);
                    sink_clone.append(source);
                },
                err_fn,
            ),
            SampleFormat::I16 => self.input_device.build_input_stream(
                &self.in_cfg.clone().into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mut filtered_data: Vec<f32> = data.iter().map(|x| (*x).to_f32()).collect();
                    apply_filter(&mut filtered_data, &low_pass_cpy);
                    apply_filter(&mut filtered_data, &high_pass_cpy);
                    let filtered_data: Vec<i16> =
                        filtered_data.iter().map(|x| (*x).to_i16()).collect();
                    let source = SamplesBuffer::new(channels_cpy, sample_rate_cpy, filtered_data);
                    sink_clone.append(source);
                },
                err_fn,
            ),
            SampleFormat::U16 => self.input_device.build_input_stream(
                &self.in_cfg.clone().into(),
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let mut filtered_data: Vec<f32> = data.iter().map(|x| (*x).to_f32()).collect();
                    apply_filter(&mut filtered_data, &low_pass_cpy);
                    apply_filter(&mut filtered_data, &high_pass_cpy);
                    let filtered_data: Vec<u16> =
                        filtered_data.iter().map(|x| (*x).to_u16()).collect();
                    let source = SamplesBuffer::new(channels_cpy, sample_rate_cpy, filtered_data);
                    sink_clone.append(source);
                },
                err_fn,
            ),
        }
        .unwrap();

        // start playback
        in_stream.play()?;
        sink.sleep_until_end();
        loop {
            if self.is_finished() {
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
