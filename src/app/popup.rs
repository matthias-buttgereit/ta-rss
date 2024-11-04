use crate::feed::entry::Entry;
use ratatui_image::thread::ThreadProtocol;
use std::sync::Arc;

pub struct Popup {
    pub entry: Arc<Entry>,
    pub image: Option<ThreadProtocol>,
    pub scroll_offset: u16,
}
impl Popup {
    pub fn new(entry: Arc<Entry>) -> Self {
        if let Some(_image_url) = &entry.image_url {
            // let image = fetch_image(image_url);
            Self {
                entry,
                scroll_offset: 0,
                image: None,
            }
        } else {
            Self {
                entry,
                scroll_offset: 0,
                image: None,
            }
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
