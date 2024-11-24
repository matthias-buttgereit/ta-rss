use crate::feed::entry::Entry;
use image::DynamicImage;
use ratatui::widgets::StatefulWidget;
use ratatui_image::thread::ThreadProtocol;
use std::sync::Arc;

use super::App;

pub struct Popup {
    pub entry: Arc<Entry>,
    pub image: Option<DynamicImage>,
    pub scroll_offset: u16,
}

impl Popup {
    pub fn new(entry: Arc<Entry>, image: Option<DynamicImage>) -> Self {
        Self {
            entry,
            image,
            scroll_offset: 0,
        }
    }

    pub(crate) fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub(crate) fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }
}

impl StatefulWidget for Popup {
    type State = ThreadProtocol;

    #[expect(unused_variables)]
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        todo!()
    }
}

impl App {
    pub fn scroll_down(&mut self) {
        if let Some(popup) = self.popup.as_mut() {
            popup.scroll_down();
        }
    }

    pub fn scroll_up(&mut self) {
        if let Some(popup) = self.popup.as_mut() {
            popup.scroll_up();
        }
    }

    pub fn toggle_popup(&mut self) {
        if self.popup.is_some() {
            self.popup = None;
        } else if let Some(index) = self.list_state.selected() {
            let entry = self.all_entries[index].clone();
            self.popup = Some(Popup::new(entry, None));
        }
    }

    pub fn update_popup(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let entry = self.all_entries[index].clone();

            // if there is an image_url ...
            if let Some(image_url) = &entry.image_url {
                // ... and we already have it in the cache
                if self.image_cache.contains_key(image_url) {
                    // write image in file
                    if let Some(_image) = self.image_cache.get(image_url).unwrap().clone() {}
                } else {
                    // if it's not already in the cache, spawn a thread to download it
                    let tx = self.image_channel.0.clone();
                    let url = image_url.to_string();

                    tokio::spawn(async move {
                        let image_bytes = reqwest::get(&url).await.unwrap().bytes().await.unwrap();
                        let image = image::load_from_memory(&image_bytes).unwrap();

                        let _ = tx.send((url, image)).await;
                    });
                    self.image_cache.insert(image_url.clone(), None);
                }
            }

            self.popup = Some(Popup::new(entry, None));
        }
    }
}
