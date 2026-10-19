use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    #[allow(dead_code)]
    Mouse(MouseEvent),
    #[allow(dead_code)]
    Resize(u16, u16),
}

pub struct EventHandler {
    receiver: mpsc::Receiver<Event>,
    _handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(Duration::ZERO);

                    if event::poll(timeout).unwrap_or(false) {
                        match event::read() {
                            Ok(CrosstermEvent::Key(key)) => {
                                if sender.send(Event::Key(key)).is_err() {
                                    break;
                                }
                            }
                            Ok(CrosstermEvent::Mouse(mouse)) => {
                                if sender.send(Event::Mouse(mouse)).is_err() {
                                    break;
                                }
                            }
                            Ok(CrosstermEvent::Resize(w, h)) => {
                                if sender.send(Event::Resize(w, h)).is_err() {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }

                    if last_tick.elapsed() >= tick_rate {
                        if sender.send(Event::Tick).is_err() {
                            break;
                        }
                        last_tick = Instant::now();
                    }
                }
            })
        };

        Self {
            receiver,
            _handler: handler,
        }
    }

    pub fn next(&self) -> Result<Event, mpsc::RecvError> {
        self.receiver.recv()
    }
}
