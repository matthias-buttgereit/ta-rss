// Import the io module from the standard library.
use std::io;

// Import the App and AppResult types from the app module.
use app::{App, AppResult};
// Import the Parser trait from the clap crate.
use clap::Parser;
// Import the Event and EventHandler types from the event module.
use event::{Event, EventHandler};
// Import the handle_key_events function from the handler module.
use handler::handle_key_events;
// Import the CrosstermBackend type from the backend module of the ratatui crate.
use ratatui::{backend::CrosstermBackend, Terminal};
// Import the Tui type from the tui module.
use tui::Tui;

// Application module.
pub mod app;

// Terminal events handler module.
pub mod event;

// Widget renderer module.
pub mod ui;

// Terminal user interface module.
pub mod tui;

// Event handler module.
pub mod handler;

// Utility functions module.
pub mod utility;

// Asynchronously start the terminal user interface with the given App.
pub async fn start_tui(mut app: App) -> AppResult<()> {
    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(100);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
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
