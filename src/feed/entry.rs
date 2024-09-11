use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use std::{io::Cursor, sync::Arc};
use tokio::sync::RwLock;

use super::image::Image;

pub struct Entry {
    pub title: String,
    pub url: String,
    pub description: String,
    pub pub_date: Option<chrono::DateTime<::chrono::FixedOffset>>,
    pub source_name: Arc<String>,
    pub image: Option<Arc<RwLock<Image>>>,
}

impl Entry {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn get_image(&self) -> anyhow::Result<Arc<RwLock<Image>>> {
        match &self.image {
            None => Err(anyhow::anyhow!("Image not available.")),
            Some(image) => {
                let image = image.clone();
                let is_downloading = image.try_read()?.is_downloading.clone();
                if !*is_downloading.try_read()? {
                    let mut is_downloading_write = is_downloading.try_write().unwrap();
                    *is_downloading_write = true;
                    tokio::spawn(async move {
                        let mut image = image.write().await;
                        let url = &image.url;
                        let response = reqwest::get(url).await.unwrap();
                        let data = response.bytes().await.unwrap().to_vec();

                        let mut picker = Picker::new((6, 12));
                        picker.guess_protocol();
                        let dyn_img = image::ImageReader::new(Cursor::new(&data))
                            .with_guessed_format()
                            .unwrap()
                            .decode()
                            .unwrap();

                        let image_data: Box<dyn StatefulProtocol> =
                            picker.new_resize_protocol(dyn_img);

                        image.data = image_data;
                    });
                }
                Ok(self.image.clone().unwrap())
            }
        }
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
