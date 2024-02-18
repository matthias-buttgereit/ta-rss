use crate::feed::Feed;
use atom_syndication::Text;
use ratatui::widgets::ListState;
use ratatui_image::protocol::StatefulProtocol;
use reqwest::Client;
use std::error;
use tokio::sync::mpsc;

// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

// Application.
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
    pub feed_receiver: mpsc::Receiver<Feed>,
    pub image_receiver: mpsc::Receiver<Box<dyn StatefulProtocol>>,
    pub image_sender: mpsc::Sender<Box<dyn StatefulProtocol>>,
    pub current_feed_image: Option<Box<dyn StatefulProtocol>>,
}

#[derive(Debug)]
pub enum AppState {
    Popup(Feed),
    List,
}

impl App {
    pub async fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Feed>(20);
        let (img_tx, img_rx) = mpsc::channel::<Box<dyn StatefulProtocol>>(1);
        let feed_urls = Self::load();
        let client = Client::new();

        for url in feed_urls.clone() {
            fetch_and_parse_feeds(url, &tx, &client);
        }

        Self {
            running: true,
            list_state: ListState::default(),
            feeds: Vec::with_capacity(feed_urls.len() * 10),
            state: AppState::List,
            feed_urls,
            feed_receiver: rx,
            image_receiver: img_rx,
            image_sender: img_tx,
            current_feed_image: None,
        }
    }

    // Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        while let Ok(feed) = self.feed_receiver.try_recv() {
            match self.feeds.binary_search(&feed) {
                Ok(_) => {} // element already in vector @ `pos`
                Err(pos) => self.feeds.insert(pos, feed),
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

fn fetch_and_parse_feeds(url: String, tx: &mpsc::Sender<Feed>, client: &Client) {
    let tx = tx.clone();
    let client = client.clone();
    tokio::spawn(async move {
        let result = client.get(url).send().await.unwrap().bytes().await.unwrap();
        if let Ok(channel) = rss::Channel::read_from(&result[..]) {
            for mut item in channel.items {
                item.set_source(rss::Source {
                    url: channel.link.to_string(),
                    title: Some(channel.title.to_string()),
                });
                tx.send(Feed::Item(item)).await.unwrap_or_default();
            }
        } else if let Ok(feed) = atom_syndication::Feed::read_from(&result[..]) {
            for mut entry in feed.entries {
                let title = Text {
                    value: feed.title.value.to_string(),
                    ..Default::default()
                };
                entry.set_source(Some(atom_syndication::Source {
                    title,
                    ..Default::default()
                }));
                tx.send(Feed::Entry(entry)).await.unwrap_or_default();
            }
        }
    });
}
