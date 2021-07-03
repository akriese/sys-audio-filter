extern crate cpal;
extern crate anyhow;
use cpal::traits::{DeviceTrait, HostTrait};
pub use sys_audio_filter::implementations::{FilterBox, CpalMgr};

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
    enum_devices().expect("Error enumerating devices!");
    let mut filter_box: CpalMgr = CpalMgr::new().unwrap();

    //filter_box.init().expect("Error initiating the box!");

    filter_box.play().expect("Error playing the sound!");
}
