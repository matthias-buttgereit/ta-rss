use std::rc::Rc;

use ratatui_image::protocol::StatefulProtocol;
use tokio::sync::mpsc;

pub struct Feed<'a> {
    pub url: String,
    pub name: String,
    pub entries: Vec<Entry<'a>>,
    pub pub_date: Option<chrono::DateTime<::chrono::FixedOffset>>,
}

pub struct Entry<'a> {
    pub title: String,
    pub url: String,
    pub description: String,
    pub pub_date: String,
    pub image: Option<Rc<dyn StatefulProtocol>>,
    pub source_name: &'a str,
}

impl<'a> Feed<'a> {
    pub fn pub_date(&self) -> Option<chrono::DateTime<::chrono::FixedOffset>> {
        todo!();
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pub_date_string(&self) -> &str {
        todo!();
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn fetch_and_parse_feeds(_url: &str, _tx: &mpsc::Sender<Feed>) {
        todo!();
    }
}

impl<'a> Entry<'a> {
    pub fn new(
        _title: String,
        _description: String,
        _pub_date: String,
        _source_name: &'a str,
    ) -> Self {
        todo!();
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
        match &self.image {
            Some(image_ref) => Some(image_ref.clone()),
            None => None,
        }
    }

    pub fn source_name(&self) -> &str {
        self.source_name
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

impl<'a> PartialOrd for Entry<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> PartialEq for Entry<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.pub_date == other.pub_date && self.title == other.title
    }
}

impl<'a> Eq for Entry<'a> {}

impl<'a> Ord for Entry<'a> {
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
