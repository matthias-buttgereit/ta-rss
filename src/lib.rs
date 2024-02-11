use app::{App, AppResult};
use atom_syndication::Feed;
use clap::Parser;
use event::{Event, EventHandler};
use futures::future::join_all;
use handler::handle_key_events;
use ratatui::{backend::CrosstermBackend, Terminal};
use rss::Channel;
use std::io;
use std::str::FromStr;
use tui::Tui;

pub mod app;
pub mod event;
pub mod handler;
pub mod tui;
pub mod ui;

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

async fn fetch_feed_string(url: &str) -> anyhow::Result<String> {
    let response = reqwest::get(url).await?;
    Ok(response.text().await?)
}

pub async fn fetch_and_parse_feed(url: &str) -> Result<FeedType, String> {
    let feed_string = fetch_feed_string(url).await.expect("Failed to fetch feed");

    // Try parsing as RSS
    if let Ok(rss_channel) = Channel::from_str(&feed_string) {
        Ok(FeedType::Rss(rss_channel))
    } else if let Ok(atom_feed) = Feed::from_str(&feed_string) {
        Ok(FeedType::Feed(atom_feed))
    } else {
        Err("Failed to parse as RSS or Atom feed".to_string())
    }
}

pub async fn load_feed_titles(urls: &[String]) -> Vec<String> {
    let feeds = join_all(urls.iter().map(|url| fetch_and_parse_feed(url))).await;

    let mut titles = vec![];

    for feed in feeds {
        match feed.unwrap() {
            FeedType::Rss(channel) => channel.items().iter().for_each(|item| {
                titles.push(item.title().unwrap().to_string());
            }),
            FeedType::Feed(feed) => feed.entries().iter().for_each(|entry| {
                titles.push(entry.title().to_string());
            }),
        }
    }

    titles
}

#[derive(Debug)]
pub enum FeedType {
    Rss(Channel),
    Feed(Feed),
}
