use std::sync::Arc;

use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use tokio::sync::mpsc::Receiver;

#[derive(Default)]
pub struct Entry {
    pub title: String,
    pub url: String,
    pub description: String,
    pub pub_date: Option<chrono::DateTime<::chrono::FixedOffset>>,
    pub source_name: Arc<String>,
    pub image_url: Option<String>,
    pub image: Option<Arc<dyn StatefulProtocol>>,
    pub image_recv: Option<Receiver<Arc<dyn StatefulProtocol>>>,
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
