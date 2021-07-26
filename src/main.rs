use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
//use tui::event::{Event, Events};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset},
    Terminal,
};
//use ctrlc::set_handler as set_ctrlc_handler;

mod platforms;
mod events;
#[cfg(target_os = "linux")]
pub use platforms::linux::PaMgr as Manager;
#[cfg(target_os = "windows")]
pub use platforms::windows::CpalMgr as Manager;
pub use platforms::{get_max_freq, FilterBox, SpectrumAnalyzer, DEFAULT_MIN_FREQ};
pub use events::{Event, EventHandler};

#[allow(dead_code)]
fn get_input() -> String {
    let mut inp = String::new();
    std::io::stdin()
        .read_line(&mut inp)
        .expect("Error reading terminal input!");
    inp.trim().to_string()
}

#[allow(dead_code)]
fn manage_box(filter_box: Arc<Manager>) {
    let min_freq = DEFAULT_MIN_FREQ;
    let max_freq = get_max_freq(filter_box.sample_rate);

    // use Ctrl+C handler to interrupt infinite sleeping loop
    //let ctrl_c_clone = filter_box.clone();
    //set_ctrlc_handler(move || {
    //ctrl_c_clone.finish();
    //println!("Keyboard Interrupt received!");
    //})
    //.expect("Error setting Ctrl+C handler");

    let mut cutoff_low = max_freq;
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
                        max_freq
                    } else {
                        let old_val = cutoff_low;
                        match command[1] as char {
                            '+' | '-' => (old_val + val).max(min_freq).min(max_freq),
                            _ => val.max(min_freq).min(max_freq),
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
                            '+' | '-' => (old_val + val).max(min_freq).min(max_freq),
                            _ => val.max(min_freq).min(max_freq),
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

struct FilterApp {
    data: Vec<(f64,f64)>,
    window: [f64; 2],
    bin_step: f32,
    spectrum_analyzer: Arc<Mutex<SpectrumAnalyzer>>,
}

impl FilterApp {
    fn new(spectrum_analyzer: Arc<Mutex<SpectrumAnalyzer>>) -> FilterApp {
        FilterApp {
            data: vec![(0.0, 0.0); 1000],
            window: [0f64, 20000f64],
            bin_step: spectrum_analyzer.lock().unwrap().bin_step(),
            spectrum_analyzer: spectrum_analyzer.clone(),
        }
    }

    fn update(&mut self) {
        self.data = self.spectrum_analyzer
            .lock()
            .unwrap()
            .get_spectrum()
            .iter()
            .enumerate()
            .map(|(f, s)| ((f as f32 * self.bin_step * 4.0) as f64, *s as f64))
            .collect();
    }
}


fn main() -> Result<(), anyhow::Error> {
    let filter_box = Arc::new(Manager::new().unwrap());
    let spectrum_analyzer = Arc::new(Mutex::new(SpectrumAnalyzer::new(2048, filter_box.sample_rate(), 2048)));

    let filter_box_cln = filter_box.clone();
    let sa_cln = spectrum_analyzer.clone();

    //thread::spawn(move || manage_box(filter_box_cln));

    thread::spawn(move || filter_box.play(sa_cln).expect("Error playing the sound!"));

    // Terminal initialization
    let stdout = std::io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = EventHandler::new();

    // App
    let mut app = FilterApp::new(spectrum_analyzer);

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Ratio(1, 1),
                    ]
                    .as_ref(),
                )
                .split(size);
            let x_labels = vec![
                Span::styled(
                    format!("{} Hz", app.window[0]),
                    Style::default().add_modifier(Modifier::BOLD)
                ),
                Span::raw(format!("{} Hz", (app.window[0] + app.window[1]) / 2.0)),
                Span::styled(
                    format!("{} Hz", app.window[1]),
                    Style::default().add_modifier(Modifier::BOLD)
                ),
            ];
            let datasets = vec![
                Dataset::default()
                    .name("Spectrum")
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(Color::Yellow))
                    .data(&app.data)
            ];

            let chart = Chart::new(datasets)
                .block(
                    Block::default()
                        .title(Span::styled(
                            "Spectrum Display",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::ITALIC),
                        ))
                        .borders(Borders::ALL),
                )
                .x_axis(
                    Axis::default()
                        .title("Frequency Strength")
                        .style(Style::default().fg(Color::Gray))
                        .labels(x_labels.clone())
                        .bounds(app.window),
                )
                .y_axis(
                    Axis::default()
                        .title("Frequency")
                        .style(Style::default().fg(Color::Gray))
                        .labels(x_labels)
                        .bounds([0.0, 1.0]),
               );

            f.render_widget(chart, chunks[0]);
        })?;

        match events.next()? {
            Event::Input(input) => {
                if input == Key::Char('q') {
                    filter_box_cln.finish();
                    break;
                }
            }
            Event::Tick => {
                app.update();
            }
        };
    }


    // wait a little, otherwise filter_box.drop() will possibly not be called
    thread::sleep(Duration::from_millis(100));
    Result::Ok(())
}
