#[cfg(target_os = "windows")]
#[link(name = "C:/Program Files/JACK2/lib/libjack64")]
extern "C" {}
//use jack::Client

fn main() {
    let (client, status) =
        jack::Client::new("rust_jack_sine", jack::ClientOptions::NO_START_SERVER).unwrap();
    println!("{}'s status is {:?}'", client.name(), status);

    let in_port = client.register_port("audio_in", jack::AudioIn);
    let out_port = client.register_port("audio_out", jack::AudioOut);
    jack::
}
