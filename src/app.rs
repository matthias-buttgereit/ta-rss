use std::error;

use ratatui::widgets::ListState;

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
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub async fn new() -> Self {
        Self {
            running: true,
            list_state: ListState::default().with_selected(Some(0)),
            test_feeds: load_feed_titles().await,
            popup_enabled: false,
            feeds: vec![],
            state: AppState::Loading,
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
