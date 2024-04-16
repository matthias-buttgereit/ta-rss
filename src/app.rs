use crate::feed::{check_url, Entry, Feed};
use ratatui_image::protocol::StatefulProtocol;
use serde::{Deserialize, Serialize};
use std::error;
use tokio::sync::mpsc;

// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;
pub type ImageData = (String, Box<dyn StatefulProtocol>);

pub struct App<'a> {
    pub running: bool,
    feed_urls: Vec<String>,
    pub popup: Option<&'a Entry>,
    pub feeds: Vec<Feed>,
    pub all_entries: Vec<&'a Entry>,
    pub list_state: ratatui::widgets::ListState,
    feed_receiver: mpsc::Receiver<Feed>,
}

impl<'a> App<'a> {
    pub async fn new() -> Self {
        let urls = load_config().unwrap_or_default();
        let channel_size = (urls.len() + 1) * 10;
        let (tx, rx) = mpsc::channel(channel_size);
        Feed::fetch_and_parse_feeds(&urls, tx).await;

        Self {
            running: true,
            feed_urls: load_config().unwrap_or_default(),
            popup: None,
            feeds: Vec::new(),
            all_entries: Vec::new(),
            list_state: ratatui::widgets::ListState::default(),
            feed_receiver: rx,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn tick(&mut self) {
        if let Ok(feed) = self.feed_receiver.try_recv() {
            self.feeds.push(feed);
            self.list_state.select(Some(0));
        }

        let entries = vec![];
        for feed in &self.feeds {
            for _entry in feed.entries() {}
        }
        self.all_entries = entries;
    }

    pub async fn add_feed(&mut self, url: &str) -> anyhow::Result<String> {
        let title = check_url(url).await?;

        self.feed_urls.push(url.to_string());
        save_config(&self.feed_urls)?;

        Ok(title)
    }

    pub(crate) fn select_previous(&mut self) {
        if let Some(index) = self.list_state.selected() {
            self.list_state.select(Some((index-1) % 2))
        }
    }

    pub(crate) fn select_next(&mut self) {
        if let Some(index) = self.list_state.selected() {
            self.list_state.select(Some((index+1) % 2))
        }
    }

    pub fn print_feeds(&self) {
        if self.feed_urls.is_empty() {
            println!("No feeds added yet. Add one with 'ta-rss add <url>'");
            return;
        }

        let mut output = String::new();
        for url in &self.feed_urls {
            output.push_str(&format!("{}\n", url));
        }
        print!("{output}");
    }

    pub fn remove_feed(&mut self, _url: &str) -> anyhow::Result<String> {
        Ok("Ok".to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    feed_urls: Vec<String>,
}

fn load_config() -> anyhow::Result<Vec<String>> {
    let config_file = "feeds.json";

    // Read the entire JSON file
    let config_data = std::fs::read_to_string(config_file).unwrap_or("".to_string());

    // Parse the JSON string into Config struct
    let config: Config = serde_json::from_str(&config_data).unwrap_or(Config {
        feed_urls: Vec::new(),
    });

    // Extract only the urls field from the Config struct
    Ok(config.feed_urls)
}

fn save_config(urls: &[String]) -> anyhow::Result<()> {
    let config = Config {
        feed_urls: urls.to_vec(),
    };
    let config_file = "feeds.json";

    // Convert the Config struct into a JSON formatted string
    let config_data = serde_json::to_string_pretty(&config).expect("Failed to serialize JSON.");

    // Write the resulting JSON string to the file
    std::fs::write(config_file, config_data).expect("Failed to write file.");

    Ok(())
}
