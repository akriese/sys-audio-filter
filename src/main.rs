extern crate cpal;
extern crate anyhow;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use cpal::{SampleFormat};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rodio::{Sample, Sink, OutputStream, buffer::SamplesBuffer};
use ctrlc;
pub use sys_audio_filter::implementations::{InputStreamWrapper, StreamConfig};

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

    let out_config = out_device.default_output_config()?;
    println!("Default output config: {:?}", out_config);

    let err_fn = |err| eprintln!("an error occurred on either audio stream: {}", err);

    let in_conf = StreamConfig{
        sample_rate: in_config.sample_rate().0,
        channels: in_config.channels()
    };
    let sink = Arc::new(Sink::try_new(&stream_handle).expect("couldnt build sink"));
    let sink_clone = sink.clone();

    let in_stream =
        match in_config.sample_format() {
            SampleFormat::F32 =>
                in_device.build_input_stream(&in_config.into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    put_to_sink::<f32>(data, &sink_clone, in_conf.channels, in_conf.sample_rate);
                }, err_fn),
            SampleFormat::I16 =>
                in_device.build_input_stream(&in_config.into(),
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    put_to_sink::<i16>(data, &sink_clone, in_conf.channels, in_conf.sample_rate);
                }, err_fn),
            SampleFormat::U16 =>
                in_device.build_input_stream(&in_config.into(),
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    put_to_sink::<u16>(data, &sink_clone, in_conf.channels, in_conf.sample_rate);
                }, err_fn),
        }
        .unwrap();

    // use Ctrl+C handler to interrupt infinite sleeping loop
    let game_over: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let game_over_clone = game_over.clone();
    ctrlc::set_handler(move || {
        game_over_clone.store(true, Ordering::Relaxed);
        println!("Keyboard Interrupt received!");
    }).expect("Error setting Ctrl+C handler");

    // start playback
    in_stream.play()?;
    sink.sleep_until_end();

    // wait in an infinite loop and wait for Keyboard Interrupt
    loop {
        if game_over.load(Ordering::Relaxed) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    // delete stream instance
    drop(in_stream);
    Result::Ok(())
}

fn put_to_sink<R>(data: &[R], sink: &Arc<Sink>, channels: u16, sample_rate: u32)
where
    R: Sample + Send + 'static, // idk why, but this only works with the 'static
{
    // create new source from data and stream configuration
    //let source = InputStreamWrapper::new(data.to_vec(), in_conf);

    // apparently, there already is an alternative for our Wrapper:
    let source = SamplesBuffer::new(channels, sample_rate, data);
    // put source into sink clone (pointing to the original sink)
    sink.append(source);

    //This is an alternative, which has worse quality than using the sink
    //stream_handle.play_raw(source).expect("Error while playbacking!");
}

fn main() {
    enum_devices().expect("Well, something went wrong apparently...");
    forward_input_to_output();
}
