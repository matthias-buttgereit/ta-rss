use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};
use std::error;

use crate::utility::{load_feed_titles, FeedType};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// counter
    pub test_feeds: Vec<String>,
    /// list state
    pub list_state: ListState,
    /// popup
    pub popup_enabled: bool,
    /// feeds
    pub feeds: Vec<FeedType>,
    /// state of the app
    /// #[allow(dead_code)]
    pub state: AppState,
    /// feed urls
    pub feed_urls: FeedURLs,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            test_feeds: vec![],
            list_state: ListState::default().with_selected(Some(0)),
            popup_enabled: false,
            feeds: vec![],
            state: AppState::Loading,
            feed_urls: FeedURLs {feeds: vec![]},
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub async fn new() -> Self {
        let feed_urls = FeedURLs::load();

        Self {
            running: true,
            list_state: ListState::default().with_selected(Some(0)),
            test_feeds: load_feed_titles().await,
            popup_enabled: false,
            feeds: vec![],
            state: AppState::Loading,
            feed_urls: feed_urls,
        }
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn select_previous(&mut self) {
        if let Some(index) = self.list_state.selected() {
            self.list_state.select(Some(
                (index + self.test_feeds.len() - 1) % self.test_feeds.len(),
            ));
        }
    }

    pub fn select_next(&mut self) {
        if let Some(index) = self.list_state.selected() {
            self.list_state
                .select(Some((index + 1) % self.test_feeds.len()));
        }
    }

    pub async fn add_feed(&mut self, url: &str) {
        self.feed_urls.feeds.push(url.to_string());
        self.feed_urls.save();
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

#[derive(Serialize, Deserialize, Debug)]
pub struct FeedURLs {
    pub feeds: Vec<String>,
}
impl FeedURLs {
    fn load() -> Self {
        let feeds = match std::fs::read_to_string("feeds.json") {
            Ok(valid_content) => {
                serde_json::from_str(&valid_content).unwrap()
            },
            Err(_) => Vec::new(),
        };

        Self { feeds }
    }

    fn save(&self) {
        let content = serde_json::to_string(&self.feeds).unwrap();
        std::fs::write("feeds.json", content).unwrap();
    }
}
