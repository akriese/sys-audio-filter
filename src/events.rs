/// The following Event and EventHandler ARE TAKEN FROM tui/examples/utils!
/// A small event handler that wrap termion input and tick events. Each event
/// type is handled in its own thread and returned to a common `Receiver`
use termion::event::Key;
use std::sync::Arc;
use std::thread;
use std::io;
use std::sync::{mpsc, atomic::{Ordering, AtomicBool}};
use std::time::Duration;
use termion::input::TermRead;

pub enum Event<I> {
    Input(I),
    Tick,
}

pub struct EventHandler {
    rx: mpsc::Receiver<Event<Key>>,
    ignore_exit_key: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Copy)]
pub struct EventHandlerConfig {
    pub exit_key: Key,
    pub tick_rate: Duration,
}

impl Default for EventHandlerConfig {
    fn default() -> EventHandlerConfig {
        EventHandlerConfig {
            exit_key: Key::Char('q'),
            tick_rate: Duration::from_millis(25),
        }
    }
}

impl EventHandler {
    pub fn new() -> EventHandler {
        EventHandler::with_config(EventHandlerConfig::default())
    }

    pub fn with_config(config: EventHandlerConfig) -> EventHandler {
        let (tx, rx) = mpsc::channel();
        let ignore_exit_key = Arc::new(AtomicBool::new(false));
        let tx_cln = tx.clone();
        let ignore_exit_key_cln = ignore_exit_key.clone();
        thread::spawn(move || {
            for evt in io::stdin().keys() {
                if let Ok(key) = evt {
                    if let Err(err) = tx_cln.send(Event::Input(key)) {
                        eprintln!("{}", err);
                        return;
                    }
                    if !ignore_exit_key_cln.load(Ordering::Relaxed) && key == config.exit_key {
                        return;
                    }
                }
            }
        });

        thread::spawn(move || loop {
            if tx.send(Event::Tick).is_err() {
                break;
            }
            thread::sleep(config.tick_rate);
        });

        EventHandler {
            rx,
            ignore_exit_key,
        }
    }

    pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn disable_exit_key(&mut self) {
        self.ignore_exit_key.store(true, Ordering::Relaxed);
    }

    pub fn enable_exit_key(&mut self) {
        self.ignore_exit_key.store(false, Ordering::Relaxed);
    }
}

