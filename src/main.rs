extern crate cpal;
extern crate anyhow;
use cpal::{Sample, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn write_silence<T: Sample>(data: &mut [T], _: &cpal::OutputCallbackInfo) {
    for sample in data.iter_mut() {
        *sample = Sample::from(&0.0);
    }
}

fn enum_devices() -> Result<(), anyhow::Error> {
    let available_hosts = cpal::available_hosts();
    for host_id in available_hosts {
        println!("{}", host_id.name());
        let host = cpal::host_from_id(host_id)?;
        let default_in = host.default_input_device().map(|e| e.name().unwrap());
        let default_out = host.default_output_device().map(|e| e.name().unwrap());
        println!("Default input device {:?}", default_in);
        println!("Default output device {:?}", default_out);
        let devices = host.devices()?;
        for (device_index, device) in devices.enumerate() {
            println!("{}. \"{}\"", device_index+1, device.name()?);
        }
    }

    Result::Ok(())
}

fn forward_input_to_output() -> Result<(), anyhow::Error> {
    let host = cpal::default_host();
    let in_device = host.input_devices()?
        .find(|x| x.name().map(|y| y == "CABLE Output (VB-Audio Virtual Cable)")
        .unwrap_or(false))
        .expect("Failed to find input device!");

    let out_device = host.output_devices()?
        .find(|x| x.name().map(|y| y == "Lautsprecher (Realtek(R) Audio)")
        .unwrap_or(false))
        .expect("Failed to find input device!");

    let (_stream, stream_handle) = OutputStream::try_from_device(&out_device)?;
    let in_config = in_device.default_input_config()?;
    println!("Default input config: {:?}", in_config);

    let out_config = out_device.default_input_config()?;
    println!("Default input config: {:?}", out_config);

    let err_fn = |err| eprintln!("an error occurred on either audio stream: {}", err);

    let mut supported_configs_range = in_device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let sample_format = supported_config.sample_format();

    // vllt buffer benutzen
    let mut data_vec: Vec<f32> = Vec::new();

    let in_stream = in_device
        .build_input_stream(&in_config.into(), move |data: &[T], _: &cpal::InputCallbackInfo| {
            get_in_stream::<f32>(data, &mut data_vec)
        },
        err_fn)
        .unwrap();
    let out_stream = out_device
        .build_output_stream(&out_config.into(), move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_in_stream::<f32>(data, &data_vec)
        }, err_fn)
        .unwrap();
    //let in_stream = match sample_format {
        //SampleFormat::F32 => in_device.build_input_stream(&in_config.into(), get_in_stream::<f32>, err_fn),
        //SampleFormat::I16 => in_device.build_input_stream(&in_config.into(), get_in_stream::<i16>, err_fn),
        //SampleFormat::U16 => in_device.build_input_stream(&in_config.into(), get_in_stream::<u16>, err_fn),
    //}.unwrap();

    //let out_stream = match sample_format {
        //SampleFormat::F32 => out_device.build_output_stream(&out_config.into(), write_in_stream::<f32>, err_fn),
        //SampleFormat::I16 => out_device.build_output_stream(&out_config.into(), write_in_stream::<i16>, err_fn),
        //SampleFormat::U16 => out_device.build_output_stream(&out_config.into(), write_in_stream::<u16>, err_fn),
    //}.unwrap();

    in_stream.play()?;
    out_stream.play()?;

    std::thread::sleep(std::time::Duration::from_secs(3));
    drop(in_stream);
    drop(out_stream);

    Result::Ok(())
}

fn get_in_stream<T>(input: &[T], data_vec: &mut &[T])
where
    T: Sample,
{
    for &sample in input.iter() {
        let sam: T = cpal::Sample::from(&sample);
        data_vec.push(sam);
    }
}

fn write_in_stream<T: Sample>(data: &mut [T], src: &[T]) {
    for sample in data.iter_mut() {
        *sample = Sample::from(&0.0);
    }
}

fn main() {
    enum_devices().expect("Well, something went wrong apparently...");
    forward_input_to_output();
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let sample_format = supported_config.sample_format();
    let config = supported_config.into();
    let stream = match sample_format {
        SampleFormat::F32 => device.build_output_stream(&config, write_silence::<f32>, err_fn),
        SampleFormat::I16 => device.build_output_stream(&config, write_silence::<i16>, err_fn),
        SampleFormat::U16 => device.build_output_stream(&config, write_silence::<u16>, err_fn),
    }.unwrap();
    stream.play().unwrap();
}
