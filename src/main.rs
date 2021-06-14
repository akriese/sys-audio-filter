extern crate cpal;
extern crate anyhow;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rodio::{Sink, OutputStream};
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

    let in_conf = StreamConfig{sample_rate: in_config.sample_rate().0, channels: in_config.channels()};
    let sink = Arc::new(Sink::try_new(&stream_handle).expect("couldnt build sink"));
    let sink_clone = sink.clone();

    let in_stream = in_device
        .build_input_stream(&in_config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // create new source from data and stream configuration
                let source = InputStreamWrapper::new(data.to_vec(), in_conf);
                // put source into sink clone (pointing to the original sink)
                sink_clone.append(source);

                //This is an alternative, which has worse quality than using the sink
                //stream_handle.play_raw(source).expect("Error while playbacking!");
            },
            err_fn)
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

fn main() {
    enum_devices().expect("Well, something went wrong apparently...");
    forward_input_to_output();
}
