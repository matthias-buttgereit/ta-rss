use crate::feed::{check_url, Entry, Feed};
use ratatui_image::protocol::StatefulProtocol;
use serde::{Deserialize, Serialize};
use std::{error, rc::Rc, sync::Arc};
use tokio::sync::mpsc;

const CONFIG_FILE_NAME: &str = "feeds.json";

pub type ImageData = (String, Box<dyn StatefulProtocol>);

pub struct App {
    pub running: bool,
    feed_urls: Vec<String>,
    pub popup: Option<Arc<Entry>>,
    pub feeds: Vec<Feed>,
    pub all_entries: Vec<Arc<Entry>>,
    pub list_state: ratatui::widgets::ListState,
    feed_receiver: mpsc::Receiver<Feed>,
}

impl App {
    pub async fn new() -> Self {
        let urls_list = load_config().unwrap_or_default();
        let (tx, rx) = mpsc::channel(urls_list.len());
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
            if self.list_state.selected().is_none() {
                self.list_state.select(Some(0));
            }

            if let Some(new_feed) = self.feeds.last() {
                for entry in new_feed.entries.iter() {
                    self.all_entries.push(entry.clone());
                }

                self.sort_entries();
            }
        }
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
            let new_index: usize;

            if index == 0 {
                new_index = self.all_entries.len() - 1
            } else {
                new_index = index - 1
            }

            self.list_state.select(Some(new_index));

            if self.popup.is_some() {
                self.popup = Some(self.all_entries[new_index].clone())
            }
        }
    }

    pub(crate) fn select_next(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let new_index: usize;

            if index == self.all_entries.len() - 1 {
                new_index = 0
            } else {
                new_index = index + 1
            }

            self.list_state.select(Some(new_index));

            if self.popup.is_some() {
                self.popup = Some(self.all_entries[new_index].clone())
            }
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
            return Err(anyhow::anyhow!(
                "No feeds added yet. Add one with 'ta-rss add <url>'"
            ));
        }

        if !self.feed_urls.contains(&url.to_string()) {
            return Err(anyhow::anyhow!("Failed to remove feed"));
        }

        self.feed_urls.retain(|x| !x.eq(&url.to_string()));
        save_config(&self.feed_urls);
        Ok(format!("Removed feed: {}", url))
    }

    pub(crate) fn toggle_popup(&mut self) {
        if self.popup.is_some() {
            self.popup = None
        } else {
            if let Some(index) = self.list_state.selected() {
                self.popup = Some(self.all_entries[index].clone())
            }
        }
    }

    fn sort_entries(&mut self) {
        self.all_entries.sort_by(|a, b| {
            b.pub_date
                .unwrap_or_default()
                .cmp(&a.pub_date.unwrap_or_default())
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Config {
    feed_urls: Vec<String>,
}

fn load_config() -> anyhow::Result<Vec<String>> {
    let config_as_json = std::fs::read_to_string(CONFIG_FILE_NAME)?;
    let config: Config = serde_json::from_str(&config_as_json)?;
    Ok(config.feed_urls)
}

fn save_config(urls: &[String]) -> anyhow::Result<()> {
    let current_config = Config {
        feed_urls: urls.to_vec(),
    };
    let config_as_json = serde_json::to_string_pretty(&current_config)?;
    std::fs::write(CONFIG_FILE_NAME, config_as_json)?;
    Ok(())
}
