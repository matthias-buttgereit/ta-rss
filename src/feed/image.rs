use std::sync::Arc;

use image::DynamicImage;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use tokio::sync::RwLock;

pub struct Image {
    pub url: String,
    pub data: Box<dyn StatefulProtocol>,
    pub is_downloading: Arc<RwLock<bool>>,
}

impl Image {
    pub fn new(url: &str) -> Self {
        let stateful_protocol =
            Picker::new((1, 1)).new_resize_protocol(DynamicImage::new_rgba8(1, 1));
        Self {
            url: url.to_owned(),
            data: stateful_protocol,
            is_downloading: Arc::new(RwLock::new(false)),
        }
    }
}
