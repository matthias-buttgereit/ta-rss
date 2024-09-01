use crate::{
    feed::{
        entry::{check_url, Entry},
        Feed,
    },
    tui,
};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

const CONFIG_FILE_NAME: &str = "feeds.json";

pub struct App {
    pub running: bool,
    feed_urls: Vec<String>,
    pub popup: Option<Arc<Entry>>,
    pub feeds: Vec<Feed>,
    pub all_entries: Vec<Arc<Entry>>,
    pub list_state: ratatui::widgets::ListState,
    feed_receiver: mpsc::Receiver<Feed>,
    pub popup_scroll_offset: u16,
}

impl App {
    pub async fn new() -> Self {
        let urls_list = load_config().unwrap_or_default();
        let (tx, rx) = mpsc::channel(urls_list.len().max(1));
        Feed::fetch_and_parse_feeds(&urls_list, tx);
        let feed_urls = load_config().unwrap_or_default();

        Self {
            running: true,
            feed_urls,
            popup: None,
            feeds: Vec::new(),
            all_entries: Vec::new(),
            list_state: ratatui::widgets::ListState::default(),
            feed_receiver: rx,
            popup_scroll_offset: 0,
        }
    }

    pub async fn start_tui(app: Self) -> anyhow::Result<()> {
        tui::start_tui(app).await
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn tick(&mut self) {
        self.receive_feeds();
    }

    fn receive_feeds(&mut self) {
        if let Ok(feed) = self.feed_receiver.try_recv() {
            self.feeds.push(feed);
            if self.list_state.selected().is_none() {
                self.list_state.select_first();
            }

            if let Some(new_feed) = self.feeds.last() {
                for entry in &new_feed.entries {
                    self.all_entries.push(entry.clone());
                }

                self.all_entries.sort();
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
            let new_index = if index == 0 {
                self.all_entries.len() - 1
            } else {
                index - 1
            };

            self.list_state.select(Some(new_index));

            if self.popup.is_some() {
                self.popup_scroll_offset = 0;
                self.popup = Some(self.all_entries[new_index].clone());
            }
        }
    }

    pub(crate) fn select_next(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let new_index = if index == self.all_entries.len() - 1 {
                0
            } else {
                index + 1
            };

            self.list_state.select(Some(new_index));

            if self.popup.is_some() {
                self.popup_scroll_offset = 0;
                self.popup = Some(self.all_entries[new_index].clone());
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
            output.push_str(&format!("{url}\n"));
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
        let _ = save_config(&self.feed_urls);
        Ok(format!("Removed feed: {url}"))
    }

    pub(crate) fn toggle_popup(&mut self) {
        self.popup_scroll_offset = 0;
        if self.popup.is_some() {
            self.popup = None;
        } else if let Some(index) = self.list_state.selected() {
            self.popup = Some(self.all_entries[index].clone());
        }
    }

    pub(crate) fn scroll_down(&mut self) {
        if self.popup.is_some() {
            self.popup_scroll_offset += 1;
        }
    }

    pub(crate) fn scroll_up(&mut self) {
        if self.popup.is_some() && self.popup_scroll_offset > 0 {
            self.popup_scroll_offset -= 1;
        }
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
