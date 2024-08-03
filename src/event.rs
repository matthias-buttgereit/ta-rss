use crossterm;
use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Paste(String),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);

        let (sender, receiver) = mpsc::unbounded_channel();

        let sender_clone = sender.clone();

        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);

            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                  _ = tick_delay => {

                    sender_clone.send(Event::Tick).unwrap_or_default();
                  }
                  Some(Ok(evt)) = crossterm_event => {

                    match evt {
                      CrosstermEvent::Key(key) => {
                        if key.kind == ratatui::crossterm::event::KeyEventKind::Press {
                          sender_clone.send(Event::Key(key)).unwrap_or_default();
                        }
                      },

                      CrosstermEvent::Mouse(mouse) => {
                        sender_clone.send(Event::Mouse(mouse)).unwrap_or_default();
                      },

                      CrosstermEvent::Resize(x, y) => {
                        sender_clone.send(Event::Resize(x, y)).unwrap_or_default();
                      },

                      CrosstermEvent::FocusLost => {},

                      CrosstermEvent::FocusGained => {},

                      CrosstermEvent::Paste(text) => {
                        sender_clone.send(Event::Paste(text)).unwrap_or_default();
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

    pub async fn next(&mut self) -> anyhow::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or(anyhow::anyhow!("Failed to receive event"))
    }
}
