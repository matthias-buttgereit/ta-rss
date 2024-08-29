use crate::app::App;
use chrono::Utc;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{block::Title, Block, BorderType, Clear, List, Paragraph, Wrap},
    Frame,
};
use ratatui_image::StatefulImage;
use std::io::Cursor;

pub fn render(app: &mut App, frame: &mut Frame) {
    let window_area = frame.area();
    let main_area = Rect {
        height: window_area.height - 1,
        ..window_area
    };
    render_list(app, frame, main_area);

    if app.feeds.is_empty() {
        render_instructions(frame, window_area);
    }

    if window_area.height > 2 {
        render_keybindings(
            app,
            frame,
            Rect {
                height: 1,
                y: window_area.height - 1,
                ..window_area
            },
        );
    }

    if app.popup.is_some() && window_area.height > 10 {
        let popup_area = Rect {
            x: (window_area.width / 2) + window_area.width % 2,
            y: window_area.y,
            width: (window_area.width / 2),
            height: window_area.height,
        };
        render_popup(app, frame, popup_area);
    }
}

fn render_instructions(frame: &mut Frame<'_>, window_area: Rect) {
    let instructions = Paragraph::new("Add feeds by running `ta-rss add <url>`")
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    let y = (window_area.height - 1) / 2;
    frame.render_widget(instructions, Rect { y, ..window_area });
}

fn render_keybindings(_app: &mut App, frame: &mut Frame, area: Rect) {
    let keybindings = "↑↓: Navigate List | Space: Open Selected Feed | Q: Quit".to_string();
    frame.render_widget(Line::raw(keybindings), area);
}

fn render_popup(app: &mut App, frame: &mut Frame, area: Rect) {
    let Some(entry) = &app.popup else { return };
    let content_width = area.width - 4;

    let date = get_age(entry.pub_date);
    let source = {
        let mut source = entry.source_name().to_owned();
        let source_len = area.width as usize - (date.len() + 4);
        source.truncate(source_len);
        source
    };

    // title
    let title = Paragraph::new(entry.title()).wrap(Wrap { trim: true });
    let title_height = title.line_count(content_width) as u16;
    let title_area = Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: content_width,
        height: title_height,
    };

    // image
    let image: Option<String> = None; // &entry.image;
    let mut image_area = Rect::default();
    let mut y_coordinate = title_area.y + title_height + 1;
    if image.is_some() {
        image_area = Rect {
            x: area.x + 2,
            y: y_coordinate,
            width: area.width - 4,
            height: (area.width - 4) / 4, // TODO clamp height to not overflow in short terminals
        };
    }

    // description
    let description = Cursor::new(entry.description());
    let description = html2text::from_read(description, content_width as usize);
    let description = Paragraph::new(description);
    let description_height = description.line_count(content_width) as u16;
    let max_description_height = area.height - y_coordinate - 2;
    let max_offset = description_height.saturating_sub(max_description_height);
    if app.popup_scroll_offset > max_offset {
        app.popup_scroll_offset = max_offset;
    }
    let description = description.scroll((app.popup_scroll_offset, 0));
    let description_area = Rect {
        x: area.x + 2,
        y: y_coordinate,
        width: area.width - 4,
        height: description_height.min(max_description_height),
    };

    let popup_height = title_area.height + description_area.height + image_area.height + 4;
    let popup_area = Rect {
        height: popup_height,
        ..area
    };

    let block = Block::bordered()
        .title(source)
        .title(Title::from(date).alignment(Alignment::Right));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);

    frame.render_widget(title, title_area);
    if let Some(image) = image {
        let _sf_image = StatefulImage::new(None);
        let _image = &mut image.clone();
        //frame.render_stateful_widget(sf_image, image_area, image);

        y_coordinate = image_area.y + image_area.height + 1;
    }

    frame.render_widget(
        description,
        Rect {
            y: y_coordinate,
            width: description_area.width + 1,
            ..description_area
        },
    );

    frame.render_widget(
        Paragraph::new(" O: Open in Browser ").alignment(Alignment::Right),
        Rect {
            y: popup_area.y + popup_area.height - 1,
            height: 2,
            ..description_area
        },
    );
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

fn render_list(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title("Ta-RSS")
        .border_style(Style::default())
        .border_type(BorderType::Rounded)
        .style(Style::default());

    let feed_list: List = List::new(app.all_entries.iter().map(|entry| entry.title()))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Black)
                .bg(Color::Gray),
        )
        .block(block);

    frame.render_stateful_widget(feed_list, area, &mut app.list_state);
}
