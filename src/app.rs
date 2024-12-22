pub mod cli;
pub mod config;
pub mod popup;

use crate::{
    feed::{entry::Entry, Feed},
    tui,
};
use config::Config;
use fxhash::FxHashMap;
use image::DynamicImage;
use popup::Popup;
use ratatui::widgets::ListState;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};

const CONFIG_FILE_NAME: &str = "feeds.json";

type Img = (String, DynamicImage);

pub struct App {
    pub running: bool,
    pub tick_rate: u64,
    feed_urls: Vec<String>,
    pub popup: Option<Popup>,
    pub feeds: Vec<Feed>,
    pub all_entries: Vec<Arc<Entry>>,
    pub list_state: ListState,
    feed_channel: (Sender<Feed>, Receiver<Feed>),
    pub image_cache: FxHashMap<String, Option<DynamicImage>>,
    image_channel: (Sender<Img>, Receiver<Img>),
}

impl App {
    pub fn new() -> Self {
        let feed_urls = Config::load().unwrap_or_default();
        let feed_channel = channel(feed_urls.len().max(1));
        let image_channel = channel(10);

        Self {
            running: true,
            tick_rate: 10,
            feed_urls,
            popup: None,
            feeds: Vec::new(),
            all_entries: Vec::new(),
            list_state: ListState::default(),
            feed_channel,
            image_cache: FxHashMap::default(),
            image_channel,
        }
    }

    pub async fn start_tui(&mut self) -> anyhow::Result<()> {
        Feed::fetch_and_parse_feeds(&self.feed_urls, &self.feed_channel.0);
        tui::start(self).await
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn tick(&mut self) {
        self.receive_feeds();
        self.receive_images();
    }

    fn receive_feeds(&mut self) {
        if let Ok(feed) = self.feed_channel.1.try_recv() {
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

    pub fn select_previous(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let new_index = if index == 0 {
                self.all_entries.len() - 1
            } else {
                index - 1
            };

            self.list_state.select(Some(new_index));
            if self.popup.is_some() {
                self.update_popup();
            }
        }
    }

    pub fn select_next(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let new_index = if index == self.all_entries.len() - 1 {
                0
            } else {
                index + 1
            };

            self.list_state.select(Some(new_index));
            if self.popup.is_some() {
                self.update_popup();
            }
        }
    }

    fn receive_images(&mut self) {
        if let Ok((url, image)) = self.image_channel.1.try_recv() {
            self.image_cache.insert(url, Some(image));
        }
    }
}
