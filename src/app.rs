pub mod cli;
pub mod config;
pub mod popup;

use crate::{
    event::handler::InputHandler,
    feed::{
        entry::{check_url, Entry},
        Feed,
    },
    tui,
};
use cli::Commands;
use config::Config;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use fxhash::FxHashMap;
use popup::Popup;
use ratatui_image::{protocol::StatefulProtocol, thread::ThreadImage};
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};

const CONFIG_FILE_NAME: &str = "feeds.json";

pub struct App {
    pub running: bool,
    feed_urls: Vec<String>,
    pub popup: Option<Popup>,
    pub feeds: Vec<Feed>,
    pub all_entries: Vec<Arc<Entry>>,
    pub list_state: ratatui::widgets::ListState,
    feed_channel: (mpsc::Sender<Feed>, mpsc::Receiver<Feed>),
    pub image_cache: FxHashMap<String, Box<dyn StatefulProtocol>>,
    image_channel: (
        Sender<(String, ThreadImage)>,
        Receiver<(String, ThreadImage)>,
    ),
}

impl App {
    pub fn new() -> Self {
        let feed_urls = Config::load().unwrap_or_default();
        let feed_channel = mpsc::channel(feed_urls.len().max(1));
        let image_channel = mpsc::channel(10);

        Self {
            running: true,
            feed_urls,
            popup: None,
            feeds: Vec::new(),
            all_entries: Vec::new(),
            list_state: ratatui::widgets::ListState::default(),
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

    async fn add_feed(&mut self, url: &str) -> anyhow::Result<String> {
        let title = check_url(url).await?;

        if self.feed_urls.contains(&url.to_string()) {
            return Err(anyhow::anyhow!("Feed already added"));
        }
        self.feed_urls.push(url.to_string());
        Config::save(&self.feed_urls)?;

        Ok(title)
    }

    fn select_previous(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let new_index = if index == 0 {
                self.all_entries.len() - 1
            } else {
                index - 1
            };

            self.list_state.select(Some(new_index));
            if self.popup.is_some() {
                self.popup = Some(Popup::new(self.all_entries[new_index].clone()));
            }
        }
    }

    fn select_next(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let new_index = if index == self.all_entries.len() - 1 {
                0
            } else {
                index + 1
            };

            self.list_state.select(Some(new_index));
            if self.popup.is_some() {
                self.popup = Some(Popup::new(self.all_entries[new_index].clone()));
            }
        }
    }

    fn print_feeds(&self) {
        if self.feed_urls.is_empty() {
            println!("No feeds added yet. Add one with 'ta-rss add <url>'");
            return;
        }

        let mut feed_titles = String::new();
        for url in &self.feed_urls {
            feed_titles.push_str(&format!("{url}\n"));
        }
        print!("{feed_titles}");
    }

    fn remove_feed(&mut self, url: &str) -> anyhow::Result<String> {
        if self.feed_urls.is_empty() {
            return Err(anyhow::anyhow!(
                "No feeds added yet. Add one with 'ta-rss add <url>'"
            ));
        }

        if !self.feed_urls.contains(&url.to_string()) {
            return Err(anyhow::anyhow!("Failed to remove feed"));
        }

        self.feed_urls.retain(|x| !x.eq(&url.to_string()));
        let _ = Config::save(&self.feed_urls);
        Ok(format!("Removed feed: {url}"))
    }

    fn toggle_popup(&mut self) {
        if self.popup.is_some() {
            self.popup = None;
        } else if let Some(index) = self.list_state.selected() {
            self.popup = Some(Popup::new(self.all_entries[index].clone()));
        }
    }

    pub async fn handle_cli_command(&mut self, command: Commands) -> anyhow::Result<()> {
        match command {
            Commands::Add { url } => match self.add_feed(&url).await {
                Ok(title) => println!("Added feed: {title}"),
                Err(e) => println!("Failed to add feed: {e}"),
            },
            Commands::Remove { url } => match self.remove_feed(&url) {
                Ok(title) => println!("Removed feed: {title}"),
                Err(e) => println!("Failed to remove feed: {e}"),
            },
            Commands::List => self.print_feeds(),
        }
        Ok(())
    }

    fn check_for_image(&mut self) {
        if let Some(popup) = &self.popup.as_mut() {
            if let Some(image_url) = &popup.entry.image_url {
                todo!("Check if image is already cached, if not download it.");
            }
        }
    }

    fn receive_images(&mut self) {
        if let Ok(image) = self.image_channel.1.try_recv() {}
    }
}

impl InputHandler for App {
    fn handle_key_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.quit(),
            KeyCode::Char('c' | 'C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.quit();
                }
            }
            KeyCode::Char(' ') => {
                self.toggle_popup();
            }
            KeyCode::Char('o' | 'O') => {
                if let Some(popup) = &self.popup {
                    let url = &popup.entry.url;
                    let _open_error = open::that_in_background(url);
                };
            }
            KeyCode::Esc => {
                if self.popup.is_some() {
                    self.popup = None;
                } else {
                    self.quit();
                }
            }
            KeyCode::Up => self.select_previous(),
            KeyCode::Down => self.select_next(),
            _ => {}
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: crossterm::event::MouseEvent) {
        if let Some(popup) = &mut self.popup {
            if mouse_event.kind == crossterm::event::MouseEventKind::ScrollDown {
                popup.scroll_down();
            }
            if mouse_event.kind == crossterm::event::MouseEventKind::ScrollUp {
                popup.scroll_up();
            }
        }
    }

    fn handle_paste_event(&mut self, _text: &str) {
        todo!("Paste event not implemented yet. Depends on crossterm feature 'bracketed-paste'.");
    }
}
