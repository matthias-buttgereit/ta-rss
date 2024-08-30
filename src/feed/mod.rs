pub mod entry;

use chrono::DateTime;
use entry::{get_image_url_for_atom, get_image_url_for_rss, Entry};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct Feed {
    pub _url: Arc<String>,
    pub name: Arc<String>,
    pub entries: Vec<Arc<Entry>>,
    pub _pub_date: Option<chrono::DateTime<::chrono::FixedOffset>>,
}

impl Feed {
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
        _url: Arc::new(url),
        name: Arc::new(atom_feed.title().to_string()),
        entries: Vec::new(),
        _pub_date: Some(atom_feed.updated),
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
            image_url: get_image_url_for_atom(&item),
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
        _url: Arc::new(url),
        name: Arc::new(channel.title.clone()),
        entries: Vec::new(),
        _pub_date: pub_date,
    };

    for item in channel.items {
        let image_url = get_image_url_for_rss(&item);

        let entry = Entry {
            title: item.title.unwrap_or("No Title".to_string()),
            url: item.link.unwrap_or("No URL provided".to_string()),
            description: item.description.unwrap_or("No Description".to_string()),
            pub_date: DateTime::parse_from_rfc2822(&item.pub_date.unwrap_or_default()).ok(),
            source_name: feed.name.clone(),
            image_url,
            ..Default::default()
        };
        let entry = entry;

        feed.entries.push(Arc::new(entry));
    }
    feed
}
