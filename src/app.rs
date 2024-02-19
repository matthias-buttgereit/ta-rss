use crate::feed::Feed;
use ratatui::widgets::ListState;
use ratatui_image::protocol::StatefulProtocol;
use std::error;
use tokio::sync::mpsc;

// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

// Application.
pub struct App {
    pub running: bool,
    pub list_state: ListState,
    pub feeds: Vec<Feed>,
    pub app_state: AppState,
    pub feed_urls: Vec<String>,
    pub feed_receiver: mpsc::Receiver<Feed>,
    pub image_receiver: mpsc::Receiver<Box<dyn StatefulProtocol>>,
    pub image_sender: mpsc::Sender<Box<dyn StatefulProtocol>>,
    pub current_feed_image: Option<Box<dyn StatefulProtocol>>,
}

#[derive(Debug)]
pub enum AppState {
    Popup(Feed),
    List,
    PastedLink(String),
}

impl App {
    pub async fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Feed>(20);
        let (img_tx, img_rx) = mpsc::channel::<Box<dyn StatefulProtocol>>(1);
        let feed_urls = Self::load();

        for url in feed_urls.clone() {
            Feed::fetch_and_parse_feeds(url, &tx);
        }

        Self {
            running: true,
            list_state: ListState::default(),
            feeds: Vec::with_capacity(feed_urls.len() * 10),
            app_state: AppState::List,
            feed_urls,
            feed_receiver: rx,
            image_receiver: img_rx,
            image_sender: img_tx,
            current_feed_image: None,
        }
    }

    pub fn tick(&mut self) {
        while let Ok(feed) = self.feed_receiver.try_recv() {
            if let Err(pos) = self.feeds.binary_search(&feed) {
                self.feeds.insert(pos, feed);
            }
        }

        if self.list_state.selected().is_none() && !self.feeds.is_empty() {
            self.list_state.select(Some(0));
        }

        if let Ok(image) = self.image_receiver.try_recv() {
            self.current_feed_image = Some(image);
        }
    }

    // Close all open channels before shutting down
    pub fn quit(&mut self) {
        self.image_receiver.close();
        self.feed_receiver.close();
        self.running = false;
    }

    pub fn select_previous(&mut self) {
        if let Some(index) = self.list_state.selected() {
            self.list_state
                .select(Some((index + self.feeds.len() - 1) % self.feeds.len()));
            if let AppState::Popup(_) = &self.app_state {
                self.update_selected_feed();
            }
        }
    }

    pub fn select_next(&mut self) {
        if let Some(index) = self.list_state.selected() {
            self.list_state.select(Some((index + 1) % self.feeds.len()));
            if let AppState::Popup(_) = &self.app_state {
                self.update_selected_feed();
            }
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

    fn update_selected_feed(&mut self) {
        if let Some(selected) = self.list_state.selected() {
            self.app_state = AppState::Popup(self.feeds.get(selected).unwrap().clone());
        }
    }
}
