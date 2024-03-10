use atom_syndication::Text;
use chrono::Datelike;
use reqwest::Client;
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub enum Feed {
    Item(rss::Item),
    Entry(atom_syndication::Entry),
}

impl Feed {
    fn pub_date(&self) -> Option<atom_syndication::FixedDateTime> {
        match self {
            Feed::Item(item) => Some(
                chrono::DateTime::parse_from_rfc2822(item.pub_date().unwrap_or_default())
                    .unwrap_or_default(),
            ),
            Feed::Entry(entry) => Some(*entry.updated()),
        }
    }

    pub fn source_name(&self) -> String {
        match self {
            Feed::Item(item) => match item.source() {
                Some(source) => source.title().unwrap_or(source.url()).to_string(),
                None => "a".to_string(),
            },
            Feed::Entry(entry) => match entry.source() {
                Some(source) => source.title().value.to_string(),
                None => "b".to_string(),
            },
        }
    }

    pub fn title(&self) -> String {
        match self {
            Feed::Item(item) => item.title().unwrap().to_string(),
            Feed::Entry(entry) => entry.title().to_string(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Feed::Item(item) => item.description().unwrap_or(" ").to_string(),
            Feed::Entry(entry) => entry.summary().unwrap_or(&Text::plain(" ")).to_string(),
        }
    }

    pub fn pub_date_string(&self) -> String {
        let now = chrono::offset::Local::now();
        let time = self.pub_date().unwrap_or_default();

        if time.year() == now.year() && time.month() == now.month() && time.day() == now.day() {
            time.format("%H:%M").to_string()
        } else {
            time.format("%y-%m-%d %H:%M").to_string()
        }
    }

    pub fn url(&self) -> String {
        match self {
            Feed::Item(item) => item.link().unwrap().to_string(),
            Feed::Entry(entry) => entry.id().to_string(),
        }
    }

    pub fn fetch_and_parse_feeds(url: &str, tx: &mpsc::Sender<Feed>) {
        let url = url.to_string();
        let client = Client::new();

        let tx = tx.clone();
        let client = client.clone();
        tokio::spawn(async move {
            let Ok(response) = client.get(url).send().await else {
                return;
            };
            let Ok(result_as_bytes) = response.bytes().await else {
                return;
            };
            if let Ok(channel) = rss::Channel::read_from(&result_as_bytes[..]) {
                for mut item in channel.items {
                    item.set_source(rss::Source {
                        url: channel.link.to_string(),
                        title: Some(channel.title.to_string()),
                    });
                    tx.send(Feed::Item(item)).await.unwrap_or_default();
                }
            } else if let Ok(feed) = atom_syndication::Feed::read_from(&result_as_bytes[..]) {
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

    pub fn get_image_url(&self) -> Option<String> {
        match self {
            Feed::Item(item) => {
                if let Some(media) = item.extensions().get("media") {
                    if let Some(content) = media.get("content") {
                        for ext in content {
                            if ext.attrs().contains_key("url") {
                                return Some(ext.attrs().get("url").unwrap().to_string());
                            }
                        }
                    }
                }
                None
            }
            Feed::Entry(_entry) => None,
        }
    }
}

impl PartialOrd for Feed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Feed {
    fn eq(&self, other: &Self) -> bool {
        self.pub_date() == other.pub_date() && self.source_name() == other.source_name()
    }
}

impl Eq for Feed {}

impl Ord for Feed {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.pub_date().cmp(&self.pub_date())
    }
}

pub async fn check_url(url: &str) -> anyhow::Result<String> {
    if let Ok(response) = reqwest::get(url).await {
        let result = response.bytes().await?;
        if let Ok(channel) = rss::Channel::read_from(&result[..]) {
            return Ok(channel.title);
        }
        if let Ok(feed) = atom_syndication::Feed::read_from(&result[..]) {
            return Ok(feed.title.value);
        }
    }

    let err = anyhow::Error::msg("Invalid URL");
    Err(err)
}
