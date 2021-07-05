extern crate cpal;
extern crate anyhow;
use std::sync::{Arc};
use std::thread;
pub use sys_audio_filter::implementations::{FilterBox, CpalMgr};

fn get_input() -> String {
    let mut inp = String::new();
    std::io::stdin().read_line(&mut inp).expect("Error reading terminal input!");
    inp = inp.trim().to_string();
    inp
}

fn main() {
    #[cfg(target_os = "windows")]
    let filter_box = Arc::new(CpalMgr::new().unwrap());
    let filter_box_cln = filter_box.clone();

    thread::spawn(move || {
        let mut cutoff_low = 20000.0;
        let mut cutoff_high = 2.0;
        loop {
            if filter_box_cln.is_finished() {
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
                        }
                        else {
                            let old_val = cutoff_low;
                            match command[1] as char {
                                '+' | '-' => (old_val + val).max(1.1),
                                _ => val.max(1.1),
                            }
                        };
                        filter_box_cln.set_filter(cutoff_low, false);
                    },
                    'h' => {
                        cutoff_high = if val == -1.0 {
                            1.1
                        }
                        else {
                            let old_val = cutoff_high;
                            match command[1] as char {
                                '+' | '-' => (old_val + val).max(1.1),
                                _ => val.max(1.1),
                            }
                        };
                        filter_box_cln.set_filter(cutoff_high, true);
                    },
                    //'v' => {
                        //filter_box_cln.sink.set_volume(volume);
                        //volume = match command[1] as char {
                            //'+' | '-' => (volume + val).max(0.0),
                            //_ => val.max(0.0),
                        //};
                        //sink.set_volume(volume);
                    //}
                    'q' => {
                        filter_box_cln.finish();
                        break;
                    },
                    _ => { },
                };
                println!("Low: {}, High: {}", cutoff_low, cutoff_high);
            }

            std::thread::sleep(std::time::Duration::from_millis(20));
        }
    });

    //filter_box.init().expect("Error initiating the box!");

    filter_box.play().expect("Error playing the sound!");
}
