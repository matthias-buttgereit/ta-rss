use crossterm::event::Event::{FocusGained, FocusLost, Key, Mouse, Paste, Resize};
use crossterm::event::{
    KeyCode::{Char, Down, Esc, Up},
    KeyEvent,
    KeyEventKind::Press,
    KeyModifiers, MouseEvent,
    MouseEventKind::{ScrollDown, ScrollUp},
};
use futures::{FutureExt, StreamExt};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::app::App;

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct EventHandler {
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);

        let (sender, receiver) = mpsc::unbounded_channel();

        let sender_clone = sender.clone();

        let _handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);

            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                  _ = tick_delay => sender_clone.send(Event::Tick).unwrap_or_default(),

                  Some(Ok(evt)) = crossterm_event => {
                    match evt {
                      Key(key) => {
                        if key.kind == Press {
                          sender_clone.send(Event::Key(key)).unwrap_or_default();
                        }
                      },
                      Mouse(mouse) => sender_clone.send(Event::Mouse(mouse)).unwrap_or_default(),
                      Resize(x, y) => sender_clone.send(Event::Resize(x, y)).unwrap_or_default(),
                      Paste(text) => sender_clone.send(Event::Paste(text)).unwrap_or_default(),
                      FocusLost | FocusGained => {},
                    }
                  }
                };
            }
        });

        Self { receiver }
    }

    pub async fn next(&mut self) -> anyhow::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or(anyhow::anyhow!("Failed to receive event"))
    }

    pub fn handle_key_events(app: &mut App, key_event: KeyEvent) {
        match key_event.code {
            Char('q') => app.quit(),
            Char('c' | 'C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.quit();
                }
            }
            Char(' ') => app.toggle_popup(),
            Char('o' | 'O') => {
                if let Some(popup) = &app.popup {
                    let url = &popup.entry.url;
                    let _open_error = open::that_in_background(url);
                };
            }
            Esc => {
                if app.popup.is_some() {
                    app.popup = None;
                } else {
                    app.quit();
                }
            }
            Up => app.select_previous(),
            Down => app.select_next(),
            _ => {}
        }
    }

    pub fn handle_mouse_events(app: &mut App, mouse_event: MouseEvent) {
        match mouse_event.kind {
            ScrollDown => app.scroll_down(),
            ScrollUp => app.scroll_up(),
            _ => {}
        }
    }

    pub fn handle_paste_event(_app: &mut App, _text: String) {
        todo!("Paste event not implemented yet. Depends on crossterm feature 'bracketed-paste'.");
    }

    pub fn handle_resize_event(_app: &mut App, _x: u16, _y: u16) {}
}
