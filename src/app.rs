use crate::feed::Feed;
use ratatui::widgets::ListState;
use reqwest::Client;
use std::error;
use tokio::sync::mpsc;

// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

// Application.
#[derive(Debug)]
pub struct App {
    // Is the application running?
    pub running: bool,
    // list state
    pub list_state: ListState,
    // feeds
    pub feeds: Vec<Feed>,
    // state of the app
    pub state: AppState,
    pub feed_urls: Vec<String>,
    pub receiver: mpsc::Receiver<Feed>,
}

impl App {
    // Constructs a new instance of [`App`].
    pub async fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Feed>(10);
        let feed_urls = Self::load();
        let client = Client::new();

        for url in feed_urls.clone() {
            let tx = tx.clone();
            let client = client.clone();
            tokio::spawn(async move {
                let result = client.get(url).send().await.unwrap().bytes().await.unwrap();
                if let Ok(feed) = rss::Channel::read_from(&result[..]) {
                    for item in feed.items {
                        tx.send(Feed::Item(item)).await.unwrap_or_default();
                    }
                } else if let Ok(channel) = atom_syndication::Feed::read_from(&result[..]) {
                    for entry in channel.entries {
                        tx.send(Feed::Entry(entry)).await.unwrap_or_default();
                    }
                }
            });
        }

        Self {
            running: true,
            list_state: ListState::default(),
            feeds: vec![],
            state: AppState::List(vec![]),
            feed_urls,
            receiver: rx,
        }
    }

    // Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        if let Ok(feed) = self.receiver.try_recv() {
            self.feeds.push(feed);
            if self.feeds.len() == 1 {
                self.list_state.select(Some(0));
            }
        }
    }

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
    Loaded,
    Error,
    Popup(Feed),
    List(Vec<_Feed>),
}

#[derive(Debug)]
pub struct _Feed {}
