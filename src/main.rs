extern crate cpal;
extern crate anyhow;
use cpal::{Sample, SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait};

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

fn main() {
    enum_devices().expect("Well, something went wrong apparently...");
    //enum_devices(host.devices());
    //let device = host.default_output_device().expect("no output device available");
    //let mut supported_configs_range = device.supported_output_configs()
        //.expect("error while querying configs");
    //let supported_config = supported_configs_range.next()
        //.expect("no supported config?!")
        //.with_max_sample_rate();
    //let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    //let sample_format = supported_config.sample_format();
    //let config = supported_config.into();
    //let stream = match sample_format {
        //SampleFormat::F32 => device.build_output_stream(&config, write_silence::<f32>, err_fn),
        //SampleFormat::I16 => device.build_output_stream(&config, write_silence::<i16>, err_fn),
        //SampleFormat::U16 => device.build_output_stream(&config, write_silence::<u16>, err_fn),
    //}.unwrap();
    //stream.play().unwrap();
}
