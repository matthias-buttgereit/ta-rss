use clap::Parser;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use ta_rss::app::{App, AppResult};
use ta_rss::event::{Event, EventHandler};
use ta_rss::handler::handle_key_events;
use ta_rss::tui::Tui;

#[tokio::main]
async fn main() -> AppResult<()> {
    let args = Arguments::parse();
    let mut app = App::new().await;

    if !args.add.is_empty() {
        app.add_feed(&args.add).await;
        println!("Added feed: {}", args.add);
    } else {
        start_tui(app).await?;
    }
    Ok(())
}

#[derive(Parser, Debug)]
struct Arguments {
    /// URL of the feed to add
    #[arg(short, long, default_value_t=String::from(""))]
    add: String,
}

async fn start_tui(mut app: App) -> AppResult<()> {
    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
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
