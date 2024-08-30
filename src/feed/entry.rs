use std::sync::Arc;

use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use tokio::sync::{oneshot::Receiver, RwLock};

#[derive(Default)]
pub struct Entry {
    pub title: String,
    pub url: String,
    pub description: String,
    pub pub_date: Option<chrono::DateTime<::chrono::FixedOffset>>,
    pub source_name: Arc<String>,
    pub image_url: Option<String>,
    pub image: Option<Arc<RwLock<Vec<u8>>>>,
    pub image_recv: Option<Receiver<Arc<dyn StatefulProtocol>>>,
}

impl Entry {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn _fetch_image(_url: &str) -> Arc<dyn StatefulProtocol> {
        let image_bytes = b"";
        let _b = image::load_from_memory(image_bytes).unwrap();
        let mut picker = Picker::new((8, 15));
        picker.protocol_type = picker.guess_protocol();

        //picker.new_resize_protocol(b).into();
        todo!();
    }

    pub fn _fetch_and_update_image(&self) {
        todo!();
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

pub fn get_image_url_for_rss(entry: &rss::Item) -> Option<String> {
    if let Some(media) = entry.extensions().get("media") {
        if let Some(content) = media.get("content") {
            for extension in content {
                if extension.name == "media:content" {
                    if let Some(url) = extension.attrs.get("url") {
                        return Some(url.to_owned());
                    }
                }
            }
        }
    }

    let html_content = entry.content().unwrap_or("");
    let document = scraper::Html::parse_document(html_content);
    let img_selector = scraper::Selector::parse("img").unwrap();

    for element in document.select(&img_selector) {
        if let Some(src) = element.value().attr("src") {
            return Some(src.to_owned());
        }
    }

    None
}

pub fn get_image_url_for_atom(entry: &atom_syndication::Entry) -> Option<String> {
    if let Some(media) = entry.extensions().get("media") {
        if let Some(content) = media.get("content") {
            for extension in content {
                if extension.name == "media:content" {
                    if let Some(url) = extension.attrs.get("url") {
                        return Some(url.to_owned());
                    }
                }
            }
        }
    }

    if let Some(content) = entry.content() {
        if let Some(html_content) = content.value() {
            let document = scraper::Html::parse_document(html_content);
            let img_selector = scraper::Selector::parse("img").unwrap();

            for element in document.select(&img_selector) {
                if let Some(url) = element.value().attr("src") {
                    return Some(url.to_owned());
                }
            }
        }
    }

    None
}
