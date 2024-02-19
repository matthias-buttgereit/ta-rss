use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

use crate::app::AppResult;

// Terminal events.
#[derive(Clone, Debug)]
pub enum Event {
    // Terminal tick.
    Tick,
    // Key press.
    Key(KeyEvent),
    // Mouse click/scroll.
    Mouse(MouseEvent),
    // Terminal resize.
    Resize(u16, u16),
    // Paste event
    Paste(String),
}

// Terminal event handler.
#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    // Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
    // Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
    // Event handler thread.
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    // Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        // Create an unbounded channel and get sender and receiver
        let (sender, receiver) = mpsc::unbounded_channel();
        // Clone the sender
        let _sender = sender.clone();

        // Spawn a new asynchronous task
        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);

            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();

                // Execute the following block if either the tick occurs or a crossterm event is received
                tokio::select! {
                  _ = tick_delay => {
                    // Send a tick event
                    _sender.send(Event::Tick).unwrap_or_default();
                  }
                  Some(Ok(evt)) = crossterm_event => {
                    // Match the received crossterm event
                    match evt {
                      CrosstermEvent::Key(key) => {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                          _sender.send(Event::Key(key)).unwrap_or_default();
                        }
                      },

                      CrosstermEvent::Mouse(mouse) => {
                        _sender.send(Event::Mouse(mouse)).unwrap_or_default();
                      },

                      CrosstermEvent::Resize(x, y) => {
                        _sender.send(Event::Resize(x, y)).unwrap_or_default();
                      },

                      CrosstermEvent::FocusLost => {
                      },

                      CrosstermEvent::FocusGained => {
                      },

                      CrosstermEvent::Paste(text) => {
                        _sender.send(Event::Paste(text)).unwrap_or_default();
                      },
                    }
                  }
                };
            }
        });

        Self {
            sender,
            receiver,
            handler,
        }
    }

    // Receive the next event from the handler thread.
    //
    // This function will always block the current thread if
    // there is no data available and it's possible for more data to be sent.
    pub async fn next(&mut self) -> AppResult<Event> {
        self.receiver
            .recv()
            .await
            .ok_or(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "This is an IO error",
            )))
    }
}
