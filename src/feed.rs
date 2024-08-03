use chrono::{DateTime, FixedOffset};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use reqwest::Client;
use std::{default, sync::Arc};
use tokio::sync::{mpsc, oneshot::Receiver};

pub struct Feed {
    pub url: Arc<String>,
    pub name: Arc<String>,
    pub entries: Vec<Arc<Entry>>,
    pub pub_date: Option<chrono::DateTime<::chrono::FixedOffset>>,
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
                    let feed = get_rss_feed(channel, url);
                    tx.send(feed).await.unwrap_or_default();
                } else if let Ok(atom_feed) = atom_syndication::Feed::read_from(&bytes[..]) {
                    let feed = get_atom_feed(url, atom_feed);
                    tx.send(feed).await.unwrap_or_default();
                }
            });
        }
    }
}

fn get_atom_feed(url: String, atom_feed: atom_syndication::Feed) -> Feed {
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
                Some(content) => content.value().unwrap_or("No Description").to_string(),
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
            ..Default::default()
        };

        feed.entries.push(Arc::new(entry));
    }
    feed
}

fn get_rss_feed(channel: rss::Channel, url: String) -> Feed {
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
            pub_date: DateTime::parse_from_rfc2822(&item.pub_date.unwrap_or_default()).ok(),
            source_name: feed.name.clone(),
            image_url: None,
            ..Default::default()
        };

        feed.entries.push(Arc::new(entry));
    }
    feed
}

#[derive(Default)]
pub struct Entry {
    pub title: String,
    pub url: String,
    pub description: String,
    pub pub_date: Option<chrono::DateTime<::chrono::FixedOffset>>,
    pub source_name: Arc<String>,
    pub image_url: Option<String>,
    pub image: Option<Arc<dyn StatefulProtocol>>,
    image_recv: Option<Receiver<Arc<dyn StatefulProtocol>>>,
}

impl Entry {
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

    fn fetch_image(url: &str) -> Arc<dyn StatefulProtocol> {
        let image_bytes = url.as_bytes();

        let b = image::load_from_memory(image_bytes).unwrap();
        let mut picker = Picker::new((8, 15));
        picker.protocol_type = picker.guess_protocol();

        picker.new_resize_protocol(b).into()
    }

    pub fn image(&mut self) -> Option<Arc<dyn StatefulProtocol>> {
        if let Some(receiver) = &mut self.image_recv {
            if let Ok(image) = receiver.try_recv() {
                return Some(image);
            }
        }

        if let Some(image_url) = &self.image_url {
            match &self.image {
                None => {
                    let _image = Entry::fetch_image(image_url);
                    return None;
                }
                Some(image) => return Some(image.clone()),
            }
        }
        None
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
    if let Ok(response) = reqwest::get(url).await {
        if let Ok(result) = response.bytes().await {
            if let Ok(channel) = rss::Channel::read_from(&result[..]) {
                return Ok(channel.title);
            }
            if let Ok(feed) = atom_syndication::Feed::read_from(&result[..]) {
                return Ok(feed.title.value);
            }
            Err(anyhow::anyhow!("Unable to parse feed."))
        } else {
            Err(anyhow::anyhow!("Unable to read feed."))
        }
    } else {
        Err(anyhow::anyhow!("Unable to fetch feed."))
    }
}
