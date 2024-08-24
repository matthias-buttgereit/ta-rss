pub mod app;
pub mod event;
pub mod feed;
pub mod handler;
pub mod render;
pub mod tui;

use app::App;
use clap::Parser;
use clap::Subcommand;
use event::{Event, EventHandler};
use handler::{_handle_paste_event, handle_key_events};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use tui::Tui;

pub async fn start_tui(mut app: App) -> anyhow::Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(20);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::Paste(text) => _handle_paste_event(&mut app, &text)?,
        }

        tui.draw(&mut app)?;
    }

    tui.exit()?;
    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Add { url: String },
    Remove { url: String },
    List,
}
