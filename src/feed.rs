use std::{rc::Rc, sync::Arc};

use atom_syndication::Link;
use chrono::{DateTime, FixedOffset};
use ratatui_image::protocol::StatefulProtocol;
use reqwest::Client;
use tokio::sync::mpsc;

pub struct Feed {
    pub url: Arc<String>,
    pub name: Arc<String>,
    pub entries: Vec<Arc<Entry>>,
    pub pub_date: Option<chrono::DateTime<::chrono::FixedOffset>>,
}

pub struct Entry {
    pub title: String,
    pub url: String,
    pub description: String,
    pub pub_date: Option<chrono::DateTime<::chrono::FixedOffset>>,
    pub source_name: Arc<String>,
    pub image_url: Option<String>,
}

pub struct FeedFetcher {
    sender: mpsc::UnboundedSender<Feed>,
    receiver: mpsc::UnboundedReceiver<Feed>,
    handler: tokio::task::JoinHandle<()>,
}

impl FeedFetcher {
    pub fn new(url: String) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let handler = tokio::spawn(async move {
            let client = Client::new();
            if let Ok(parsed_url) = reqwest::Url::parse(&url) {}
        });

        Self {
            sender,
            receiver,
            handler,
        }
    }
}

impl Feed {
    pub fn pub_date(&self) -> Option<chrono::DateTime<::chrono::FixedOffset>> {
        todo!();
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn entries(&self) -> &Vec<Arc<Entry>> {
        &self.entries
    }

    pub fn pub_date_string(&self) -> &str {
        todo!();
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn fetch_and_parse_feeds(urls: &[String], tx: mpsc::Sender<Feed>) {
        let client = Client::new();
        for url in urls {
            let client = client.clone();
            let url = url.clone();
            let Ok(parsed_url) = reqwest::Url::parse(&url) else {
                continue;
            };
            let tx = tx.clone();

            tokio::spawn(async move {
                let Ok(response) = client.get(parsed_url).send().await else {
                    return;
                };
                let Ok(bytes) = response.bytes().await else {
                    return;
                };

                if let Ok(channel) = rss::Channel::read_from(&bytes[..]) {
                    let pub_date = match channel.pub_date() {
                        Some(pub_date) => {
                            if let Ok(date) = chrono::DateTime::parse_from_rfc2822(pub_date) {
                                Some(date)
                            } else {
                                None
                            }
                        }
                        None => None,
                    };

                    let mut feed = Feed {
                        url: Arc::new(url),
                        name: Arc::new(channel.title.clone()),
                        entries: Vec::new(),
                        pub_date,
                    };

                    for item in channel.items {
                        let entry = Entry {
                            title: item.title.unwrap_or("No Title".to_string()),
                            url: item.link.unwrap_or("No URL provided".to_string()),
                            description: item.description.unwrap_or("No Description".to_string()),
                            pub_date: DateTime::parse_from_rfc2822(
                                &item.pub_date.unwrap_or_default(),
                            )
                            .ok(),
                            source_name: feed.name.clone(),
                            image_url: None,
                        };

                        feed.entries.push(Arc::new(entry));
                    }
                    tx.send(feed).await.unwrap_or_default();
                } else if let Ok(atom_feed) = atom_syndication::Feed::read_from(&bytes[..]) {
                    let mut feed = Feed {
                        url: Arc::new(url),
                        name: Arc::new(atom_feed.title().to_string()),
                        entries: Vec::new(),
                        pub_date: Some(atom_feed.updated),
                    };

                    for item in atom_feed.entries {
                        let url = match item.links.first() {
                            Some(link) => link.href().to_string(),
                            None => "No URL provided".to_string(),
                        };

                        let description = match item.summary() {
                            Some(text) => text.to_string(),
                            None => match item.content() {
                                Some(content) => {
                                    content.value().unwrap_or("No Description").to_string()
                                }
                                None => "No Description".to_string(),
                            },
                        };

                        let entry = Entry {
                            title: item.title.to_string(),
                            url,
                            description,
                            pub_date: item.published,
                            source_name: feed.name.clone(),
                            image_url: None,
                        };

                        feed.entries.push(Arc::new(entry));
                    }
                    tx.send(feed).await.unwrap_or_default();
                }
            });
        }
    }
}

impl Entry {
    pub fn new(
        title: String,
        description: String,
        pub_date: Option<DateTime<FixedOffset>>,
        source_name: Arc<String>,
        image_url: Option<String>,
    ) -> Self {
        Self {
            title,
            description,
            pub_date,
            url: String::new(),
            image_url,
            source_name,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn pub_date_string(&self) -> String {
        match &self.pub_date {
            Some(time) => time.to_rfc2822(),
            None => "No Publish Date".to_string(),
        }
    }

    pub fn image(&self) -> Option<Rc<dyn StatefulProtocol>> {
        todo!();
        // match &self.image {
        //     Some(image_ref) => Some(image_ref.clone()),
        //     None => None,
        // }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn source_name(&self) -> &str {
        &self.source_name
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.pub_date == other.pub_date && self.title == other.title
    }
}

impl Eq for Entry {}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.pub_date.cmp(&self.pub_date)
    }
}

pub async fn check_url(url: &str) -> anyhow::Result<String> {
    let response = reqwest::get(url).await?;
    let result = response.bytes().await?;

    if let Ok(channel) = rss::Channel::read_from(&result[..]) {
        return Ok(channel.title);
    }
    if let Ok(feed) = atom_syndication::Feed::read_from(&result[..]) {
        return Ok(feed.title.value);
    }

    let err = anyhow::Error::msg("Unable to parse feed.");
    Err(err)
}
