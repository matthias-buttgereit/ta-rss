use crate::feed::{check_url, Entry, Feed};
use ratatui_image::protocol::StatefulProtocol;
use serde::{Deserialize, Serialize};
use std::error;
use tokio::sync::mpsc;

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
        let urls_list = load_config().unwrap_or_default();
        let channel_size = (urls_list.len() + 1) * 10;
        let (tx, rx) = mpsc::channel(channel_size);
        Feed::fetch_and_parse_feeds(&urls_list, tx).await;

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

        if self.feed_urls.contains(&url.to_string()) {
            return Err(anyhow::anyhow!("Feed already added"));
        }
        self.feed_urls.push(url.to_string());
        save_config(&self.feed_urls)?;

        Ok(title)
    }

    pub(crate) fn select_previous(&mut self) {
        if let Some(index) = self.list_state.selected() {
            self.list_state.select(Some((index - 1) % 2))
        }
    }

    pub(crate) fn select_next(&mut self) {
        if let Some(index) = self.list_state.selected() {
            self.list_state.select(Some((index + 1) % 2))
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

    pub fn remove_feed(&mut self, url: &str) -> anyhow::Result<String> {
        if self.feed_urls.is_empty() {
            return Err(anyhow::anyhow!("No feeds added yet. Add one with 'ta-rss add <url>'"));
        }

        if !self.feed_urls.contains(&url.to_string()) {
            return Err(anyhow::anyhow!("Failed to remove feed"));
        }
        
        self.feed_urls.retain(|x| x.eq(&url.to_string()));
        Ok(format!("Removed feed: {}", url))
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    feed_urls: Vec<String>,
}

fn load_config() -> anyhow::Result<Vec<String>> {
    let config_file = "feeds.json";

    let config_data = std::fs::read_to_string(config_file).unwrap_or("".to_string());

    let config: Config = serde_json::from_str(&config_data).unwrap_or(Config {
        feed_urls: Vec::new(),
    });

    Ok(config.feed_urls)
}

fn save_config(urls: &[String]) -> anyhow::Result<()> {
    let config = Config {
        feed_urls: urls.to_vec(),
    };
    let config_file = "feeds.json";

    let config_data = serde_json::to_string_pretty(&config).expect("Failed to serialize JSON.");

    std::fs::write(config_file, config_data).expect("Failed to write file.");

    Ok(())
}
