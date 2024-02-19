use app::{App, AppResult};
use clap::Parser;
use event::{Event, EventHandler};
use handler::{_handle_paste_event, handle_key_events};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tui::Tui;

pub mod app;
pub mod event;
pub mod feed;
pub mod handler;
pub mod tui;
pub mod ui;

// Asynchronously start the terminal user interface with the given App.
pub async fn start_tui(mut app: App) -> AppResult<()> {
    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::Paste(text) => _handle_paste_event(&mut app, text)?,
        }
        // Render the user interface.
        tui.draw(&mut app)?;
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}

// Struct for parsing command-line arguments.
#[derive(Parser, Debug)]
pub struct Arguments {
    // URL of the feed to add
    #[arg(short, long, default_value_t=String::from(""))]
    pub add: String,
}
