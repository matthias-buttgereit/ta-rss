use crate::feed::{check_url, Feed};
use ratatui::widgets::ListState;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use rustc_hash::FxHashMap;
use std::{env, error, fs};
use tokio::sync::mpsc;

// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;
pub type ImageData = (String, Box<dyn StatefulProtocol>);

// Application.
pub struct App {
    pub running: bool,
    pub list_state: ListState,
    pub feeds: Vec<Feed>,
    pub app_state: AppState,
    pub feed_urls: Vec<String>,
    pub feed_receiver: mpsc::Receiver<Feed>,
    pub image_receiver: mpsc::Receiver<ImageData>,
    pub image_sender: mpsc::Sender<ImageData>,
    pub current_feed_image: Option<Box<dyn StatefulProtocol>>,
    pub cached_images: FxHashMap<String, Box<dyn StatefulProtocol>>,
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
        let (img_tx, img_rx) = mpsc::channel::<ImageData>(1);
        let feed_urls = Self::load();

        for url in feed_urls.iter() {
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
            cached_images: FxHashMap::default(),
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

        if let Ok((url, image)) = self.image_receiver.try_recv() {
            if let AppState::Popup(feed) = &self.app_state {
                if let Some(feed_url) = feed.get_image_url() {
                    if feed_url == url {
                        self.current_feed_image = Some(image.clone());
                    }
                }
            }
            self.cached_images.insert(url, image);
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
                self.update_displayed_feed();
            }
        }
    }

    pub fn select_next(&mut self) {
        if let Some(index) = self.list_state.selected() {
            self.list_state.select(Some((index + 1) % self.feeds.len()));
            if let AppState::Popup(_) = &self.app_state {
                self.update_displayed_feed();
            }
        }
    }

    pub async fn add_feed(&mut self, url: &str) -> anyhow::Result<String> {
        let title = check_url(url).await?;

        self.feed_urls.push(url.to_string());
        if let Err(a) = self.save() {
            return Err(anyhow::Error::msg(a.to_string()));
        }

        Ok(title)
    }

    fn load() -> Vec<String> {
        let exe_path = env::current_exe().unwrap();
        let reading_file_path = exe_path.parent().unwrap().join("feeds.json");
        match std::fs::read_to_string(reading_file_path) {
            Ok(valid_content) => match serde_json::from_str(&valid_content) {
                Ok(feed_urls) => feed_urls,
                Err(_) => Vec::new(),
            },
            Err(_) => Vec::new(),
        }
    }

    fn save(&self) -> std::io::Result<()> {
        let exe_path = env::current_exe().unwrap();
        let output_file_path = exe_path.parent().unwrap().join("feeds.json");
        let content = serde_json::to_string(&self.feed_urls).unwrap();
        fs::write(output_file_path, content)
    }

    fn update_displayed_feed(&mut self) {
        self.current_feed_image = None;
        if let Some(selected) = self.list_state.selected() {
            let displayed_feed = self.feeds.get(selected).unwrap();
            self.app_state = AppState::Popup(displayed_feed.clone());

            if let Some(feed_image_url) = displayed_feed.get_image_url() {
                if self.cached_images.contains_key(&feed_image_url) {
                    self.current_feed_image =
                        Some(self.cached_images.get(&feed_image_url).unwrap().clone());
                } else {
                    let tx = self.image_sender.clone();
                    tokio::spawn(async move {
                        let image_bytes = reqwest::get(&feed_image_url)
                            .await
                            .unwrap()
                            .bytes()
                            .await
                            .unwrap();

                        let b = image::load_from_memory(&image_bytes).unwrap();
                        let mut picker = Picker::new((8, 15));
                        picker.protocol_type = picker.guess_protocol();

                        let image = picker.new_resize_protocol(b);
                        let _result = tx.send((feed_image_url, image)).await;
                    });
                }
            }
        }
    }

    pub fn print_feeds(&self) {
        if self.feed_urls.is_empty() {
            println!("No feeds found.");
        } else {
            for url in &self.feed_urls {
                println!("{}", url);
            }
        }
    }

    pub fn remove_feed(&mut self, _url: &str) -> anyhow::Result<String> {
        todo!()
    }
}

pub struct Popup<'a> {
    pub title: &'a str,
    pub content: &'a str,
    pub timestamp: Option<String>,
    pub image: Option<Box<dyn StatefulProtocol>>,
    pub source: Option<&'a str>,
}
