use crate::feed::{check_url, Entry, Feed};
use ratatui_image::protocol::StatefulProtocol;
use serde::{Deserialize, Serialize};
use std::error;

// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;
pub type ImageData = (String, Box<dyn StatefulProtocol>);

pub struct App<'a> {
    pub running: bool,
    feed_urls: Vec<String>,
    pub popup: Option<&'a Entry<'a>>,
    pub feeds: Vec<Feed<'a>>,
    pub list_state: ratatui::widgets::ListState,
}

impl<'a> App<'a> {
    pub async fn new() -> Self {
        Self {
            running: true,
            feed_urls: load_config().unwrap_or_default(),
            popup: None,
            feeds: Vec::new(),
            list_state: ratatui::widgets::ListState::default(),
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn tick(&mut self) {
        todo!("Implement tick")
    }

    pub async fn add_feed(&mut self, url: &str) -> anyhow::Result<String> {
        let title = check_url(url).await?;

        self.feed_urls.push(url.to_string());
        save_config(&self.feed_urls)?;

        Ok(title)
    }

    pub(crate) fn select_previous(&self) {
        todo!()
    }

    pub(crate) fn select_next(&self) {
        todo!()
    }

    pub fn print_feeds(&self) {}

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
    let config_data = std::fs::read_to_string(config_file).expect("Failed to read file");

    // Parse the JSON string into Config struct
    let config: Config = serde_json::from_str(&config_data).expect("Failed to parse JSON");

    // Extract only the urls field from the Config struct
    Ok(config.feed_urls)
}

fn save_config(urls: &[String]) -> anyhow::Result<()> {
    let config = Config {
        feed_urls: urls.to_vec(),
    };
    let config_file = "feeds.json";

    // Convert the Config struct into a JSON formatted string
    let config_data = serde_json::to_string_pretty(&config).expect("Failed to serialize JSON");

    // Write the resulting JSON string to the file
    std::fs::write(config_file, config_data).expect("Failed to write file");

    Ok(())
}
