use std::rc::Rc;

use ratatui_image::protocol::StatefulProtocol;
use reqwest::Client;
use tokio::sync::mpsc;

pub struct Feed {
    pub url: String,
    pub name: String,
    pub entries: Vec<Entry>,
    pub pub_date: Option<chrono::DateTime<::chrono::FixedOffset>>,
}

pub struct Entry {
    pub title: String,
    pub url: String,
    pub description: String,
    pub pub_date: String,
    pub source_name: String,
    pub image_url: Option<String>,
}

impl Feed {
    pub fn pub_date(&self) -> Option<chrono::DateTime<::chrono::FixedOffset>> {
        todo!();
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn entries(&self) -> &Vec<Entry> {
        &self.entries
    }

    pub fn pub_date_string(&self) -> &str {
        todo!();
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub async fn fetch_and_parse_feeds(urls: &[String], tx: mpsc::Sender<Feed>) {
        let client = Client::new();
        for url in urls {
            let client = client.clone();
            let url = url.clone();
            let Ok(parsed_url) = reqwest::Url::parse(&url) else {continue};
            let tx = tx.clone();

            tokio::spawn(async move {
                let Ok(response) = client.get(parsed_url).send().await else {return};
                let Ok(bytes) = response.bytes().await else {return};

                if let Ok(channel) = rss::Channel::read_from(&bytes[..]) {
                    let pub_date = match channel.pub_date() {
                        Some(pub_date) => {
                            if let Ok(date) = chrono::DateTime::parse_from_rfc2822(pub_date) {
                                Some(date)
                            } else {
                                None
                            }
                        },
                        None => None,
                    };

                    let mut feed = Feed {
                        url: url.to_owned(),
                        name: channel.title.clone(),
                        entries: Vec::new(),
                        pub_date,
                    };

                    for item in channel.items {
                        let entry = Entry {
                            title: item.title.unwrap_or("No Title".to_string()),
                            url: item.link.unwrap_or("No URL provided".to_string()),
                            description: item.description.unwrap_or("No Description".to_string()),
                            pub_date: item.pub_date.unwrap_or_default(),
                            source_name: channel.title.clone(),
                            image_url: None,
                        };

                        feed.entries.push(entry);
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
        pub_date: String,
        source_name: String,
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

    pub fn pub_date_string(&self) -> &str {
        &self.pub_date
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
