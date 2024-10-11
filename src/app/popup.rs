use std::sync::Arc;

use crate::feed::entry::Entry;

pub struct Popup {
    pub entry: Arc<Entry>,
    pub scroll_offset: u16,
}
impl Popup {
    pub fn new(entry: Arc<Entry>) -> Self {
        Self {
            entry,
            scroll_offset: 0,
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset += 1;
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }
}
