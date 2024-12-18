use crate::feed::entry::Entry;
use chrono::Utc;
use ratatui::prelude::*;
use ratatui::{
    layout::{Alignment, Rect},
    widgets::{block::Title, Block, Clear, Paragraph, Widget, WidgetRef, Wrap},
};
use ratatui_image::thread::ThreadProtocol;
use std::sync::Arc;

use super::App;

pub struct Popup {
    pub entry: Arc<Entry>,
    pub image: Option<ThreadProtocol>,
    pub scroll_offset: u16,
}

impl Popup {
    pub fn new(entry: Arc<Entry>, image: Option<ThreadProtocol>) -> Self {
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

impl WidgetRef for Popup {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let content_width = area.width - 4;

        let date = get_age(self.entry.pub_date);
        let source = {
            let mut source = self.entry.source_name().to_owned();
            let source_len = area.width as usize - (date.len() + 4);
            source.truncate(source_len);
            source
        };

        // title
        let title = Paragraph::new(self.entry.title()).wrap(Wrap { trim: true });
        let title_height = u16::try_from(title.line_count(content_width)).unwrap();
        let title_area = Rect {
            x: area.x + 2,
            y: area.y + 2,
            width: content_width,
            height: title_height,
        };

        // image
        // let mut image_area = Rect::default();
        let mut y_coordinate = title_area.y + title_height + 1;

        if self.image.is_some() {
            // image_area = Rect {
            //     x: area.x + 2,
            //     y: y_coordinate,
            //     width: area.width - 4,
            //     height: (area.width - 4) / 4, // TODO clamp height to not overflow in short terminals
            // };
            y_coordinate += 10;
        }

        // let image_result = self.entry.get_image();

        // if image_result.is_ok() {
        //     image_area = Rect {
        //         x: area.x + 2,
        //         y: y_coordinate,
        //         width: area.width - 4,
        //         height: (area.width - 4) / 4, // TODO clamp height to not overflow in short terminals
        //     };
        //     y_coordinate += 10;
        // }

        // description
        let description = std::io::Cursor::new(self.entry.description());
        let description = html2text::from_read(description, content_width as usize).unwrap();
        let description = Paragraph::new(description);
        let description_height = u16::try_from(description.line_count(content_width)).unwrap();
        let max_description_height = area.height - y_coordinate - 2;

        let description = description.scroll((self.scroll_offset, 0));
        let description_area = Rect {
            x: area.x + 2,
            y: y_coordinate,
            width: area.width - 4,
            height: description_height.min(max_description_height),
        };

        let popup_height = title_area.height + description_area.height + 4;
        // let popup_height = popup_height + image_height;
        let popup_area = Rect {
            height: popup_height,
            ..area
        };

        #[expect(deprecated)]
        let block = Block::bordered()
            .title(source)
            .title(Title::from(date).alignment(Alignment::Right));

        Clear.render(popup_area, buf);
        block.render(popup_area, buf);
        title.render(title_area, buf);

        // render image here
        // if let Some(image) = &mut popup.image {
        //     let sf_image = ThreadImage::default().resize(Resize::Crop(None));
        //     frame.render_stateful_widget(sf_image, image_area, image);
        // }

        description.render(
            Rect {
                y: y_coordinate,
                width: description_area.width + 1,
                ..description_area
            },
            buf,
        );

        Paragraph::new(" O: Open in Browser ")
            .alignment(Alignment::Right)
            .render(
                Rect {
                    y: popup_area.y + popup_area.height - 1,
                    height: 2,
                    ..description_area
                },
                buf,
            );
    }
}

impl Widget for Popup {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let content_width = area.width - 4;

        let date = get_age(self.entry.pub_date);
        let source = {
            let mut source = self.entry.source_name().to_owned();
            let source_len = area.width as usize - (date.len() + 4);
            source.truncate(source_len);
            source
        };

        // title
        let title = Paragraph::new(self.entry.title()).wrap(Wrap { trim: true });
        let title_height = u16::try_from(title.line_count(content_width)).unwrap();
        let title_area = Rect {
            x: area.x + 2,
            y: area.y + 2,
            width: content_width,
            height: title_height,
        };

        // image
        // let mut image_area = Rect::default();
        let mut y_coordinate = title_area.y + title_height + 1;

        if self.image.is_some() {
            // image_area = Rect {
            //     x: area.x + 2,
            //     y: y_coordinate,
            //     width: area.width - 4,
            //     height: (area.width - 4) / 4, // TODO clamp height to not overflow in short terminals
            // };
            y_coordinate += 10;
        }

        // let image_result = self.entry.get_image();

        // if image_result.is_ok() {
        //     image_area = Rect {
        //         x: area.x + 2,
        //         y: y_coordinate,
        //         width: area.width - 4,
        //         height: (area.width - 4) / 4, // TODO clamp height to not overflow in short terminals
        //     };
        //     y_coordinate += 10;
        // }

        // description
        let description = std::io::Cursor::new(self.entry.description());
        let description = html2text::from_read(description, content_width as usize).unwrap();
        let description = Paragraph::new(description);
        let description_height = u16::try_from(description.line_count(content_width)).unwrap();
        let max_description_height = area.height - y_coordinate - 2;

        let description = description.scroll((self.scroll_offset, 0));
        let description_area = Rect {
            x: area.x + 2,
            y: y_coordinate,
            width: area.width - 4,
            height: description_height.min(max_description_height),
        };

        let popup_height = title_area.height + description_area.height + 4;
        // let popup_height = popup_height + image_height;
        let popup_area = Rect {
            height: popup_height,
            ..area
        };

        #[expect(deprecated)]
        let block = Block::bordered()
            .title(source)
            .title(Title::from(date).alignment(Alignment::Right));

        Clear.render(popup_area, buf);
        block.render(popup_area, buf);
        title.render(title_area, buf);

        // render image here
        // if let Some(image) = &mut popup.image {
        //     let sf_image = ThreadImage::default().resize(Resize::Crop(None));
        //     frame.render_stateful_widget(sf_image, image_area, image);
        // }

        description.render(
            Rect {
                y: y_coordinate,
                width: description_area.width + 1,
                ..description_area
            },
            buf,
        );

        Paragraph::new(" O: Open in Browser ")
            .alignment(Alignment::Right)
            .render(
                Rect {
                    y: popup_area.y + popup_area.height - 1,
                    height: 2,
                    ..description_area
                },
                buf,
            );
    }
}

fn get_age(date: Option<chrono::prelude::DateTime<chrono::prelude::FixedOffset>>) -> String {
    match date {
        None => String::new(),
        Some(date) => {
            let age = Utc::now() - date.with_timezone(&Utc);
            if age.num_weeks() > 0 {
                let plural_s = if age.num_weeks() > 1 { "s" } else { "" };
                (format!("{} week{} ago", age.num_weeks(), plural_s)).to_string()
            } else if age.num_days() > 0 {
                let plural_s = if age.num_days() > 1 { "s" } else { "" };
                (format!("{} day{} ago", age.num_days(), plural_s)).to_string()
            } else if age.num_hours() > 0 {
                let plural_s = if age.num_hours() > 1 { "s" } else { "" };
                (format!("{} hour{} ago", age.num_hours(), plural_s)).to_string()
            } else if age.num_minutes() > 0 {
                let plural_s = if age.num_minutes() > 1 { "s" } else { "" };
                (format!("{} minute{} ago", age.num_minutes(), plural_s)).to_string()
            } else {
                "Just now".to_string()
            }
        }
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

            if let Some(image_url) = &entry.image_url {
                if self.image_cache.contains_key(image_url) {
                    if let Some(_image) = self.image_cache.get(image_url).unwrap().clone() {}
                } else {
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
