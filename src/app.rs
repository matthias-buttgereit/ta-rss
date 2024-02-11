use ratatui::widgets::ListState;
use std::error;

use crate::{load_feed_titles, FeedType};

// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

// Application.
#[derive(Debug)]
pub struct App {
    // Is the application running?
    pub running: bool,
    // list state
    pub list_state: ListState,
    // popup
    pub content_popup_open: bool,
    // feeds
    pub feeds: Vec<String>,
    // state of the app
    pub state: AppState,
    // feed urls
    pub feed_urls: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            feeds: vec![],
            list_state: ListState::default().with_selected(Some(0)),
            content_popup_open: false,
            state: AppState::Loading,
            feed_urls: vec![],
        }
    }
}

impl App {
    // Constructs a new instance of [`App`].
    pub async fn new() -> Self {
        let feed_urls = Self::load();
        let feeds = load_feed_titles(&feed_urls).await;

        Self {
            running: true,
            list_state: ListState::default().with_selected(Some(0)),
            content_popup_open: false,
            feeds,
            state: AppState::Loading,
            feed_urls,
        }
    }

    // Handles the tick event of the terminal.
    pub fn tick(&self) {}

    // Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn select_previous(&mut self) {
        if let Some(index) = self.list_state.selected() {
            self.list_state
                .select(Some((index + self.feeds.len() - 1) % self.feeds.len()));
        }
    }

    pub fn select_next(&mut self) {
        if let Some(index) = self.list_state.selected() {
            self.list_state.select(Some((index + 1) % self.feeds.len()));
        }
    }

    pub async fn add_feed(&mut self, url: &str) {
        self.feed_urls.push(url.to_string());
        self.save();
    }

    fn load() -> Vec<String> {
        match std::fs::read_to_string("feeds.json") {
            Ok(valid_content) => serde_json::from_str(&valid_content).unwrap(),
            Err(_) => Vec::new(),
        }
    }

    fn save(&self) {
        let content = serde_json::to_string(&self.feed_urls).unwrap();
        std::fs::write("feeds.json", content).unwrap();
    }
}

#[derive(Debug)]
pub enum AppState {
    Loading,
    Loaded(FeedType),
    Error,
    Popup(_Feed),
    List(Vec<_Feed>),
}

#[derive(Debug)]
pub struct _Feed {}
