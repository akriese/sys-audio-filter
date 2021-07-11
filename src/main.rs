extern crate anyhow;
extern crate cpal;
use std::sync::Arc;
use std::thread;

mod platforms;
#[cfg(target_os = "linux")]
pub use platforms::linux::PaMgr as Manager;
#[cfg(target_os = "windows")]
pub use platforms::windows::CpalMgr as Manager;
pub use platforms::FilterBox;

fn get_input() -> String {
    let mut inp = String::new();
    std::io::stdin()
        .read_line(&mut inp)
        .expect("Error reading terminal input!");
    inp.trim().to_string()
}

fn manage_box(filter_box: Arc<Manager>) {
    let min_freq = 10.0;

    // use Ctrl+C handler to interrupt infinite sleeping loop
    let ctrl_c_clone = filter_box.clone();
    ctrlc::set_handler(move || {
        ctrl_c_clone.finish();
        println!("Keyboard Interrupt received!");
    })
    .expect("Error setting Ctrl+C handler");

    let mut cutoff_low = 20000.0;
    let mut cutoff_high = min_freq;

    loop {
        if filter_box.is_finished() {
            println!("Ending program...");
            break;
        }

        let inp = get_input();
        let command = inp.as_bytes();
        let mut val = -1.0;
        if command.len() > 0 {
            if command.len() > 1 {
                val = inp[1..].to_string().parse::<f32>().unwrap();
            }
            match command[0] as char {
                'l' => {
                    cutoff_low = if val == -1.0 {
                        20000.0
                    } else {
                        let old_val = cutoff_low;
                        match command[1] as char {
                            '+' | '-' => (old_val + val).max(min_freq),
                            _ => val.max(min_freq),
                        }
                    };
                    filter_box.set_filter(cutoff_low, false);
                }
                'h' => {
                    cutoff_high = if val == -1.0 {
                        min_freq
                    } else {
                        let old_val = cutoff_high;
                        match command[1] as char {
                            '+' | '-' => (old_val + val).max(min_freq),
                            _ => val.max(min_freq),
                        }
                    };
                    filter_box.set_filter(cutoff_high, true);
                }
                /*
                'v' => {
                    let volume: u16 = command[1] as u16;
                    filter_box.set_volume(volume);
                }
                */
                'q' => {
                    filter_box.finish();
                    continue;
                }
                _ => {}
            };
            println!(
                "You can hear frequencies between {}hz (highpass freq) and {}hz (lowpass freq)",
                cutoff_high, cutoff_low
            );
        }

        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

fn main() {
    let filter_box = Arc::new(Manager::new().unwrap());

    let filter_box_cln = filter_box.clone();

    thread::spawn(move || manage_box(filter_box_cln));

    filter_box.play().expect("Error playing the sound!");
}
