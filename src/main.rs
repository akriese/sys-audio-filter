use cpal::traits::HostTrait;

fn main() {
    let host = cpal::default_host();
    println!("The Host is {:?}", host.id());
    let devices = host.devices();
    for d in devices {
        println!(" {} ", d.name());
    }
}
