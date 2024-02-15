use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, Clear, List, Paragraph, Wrap},
    Frame,
};
use ratatui_image::StatefulImage;

use crate::app::{App, AppState};

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.size();

    let block = Block::bordered()
        .title("Ta-RSS")
        .border_style(Style::default())
        .border_type(BorderType::Rounded)
        .style(Style::default());

    let feed_list: List = List::new(app.feeds.iter().map(|feed| feed.title().clone()))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Black)
                .bg(Color::Gray),
        )
        .block(block);

    frame.render_stateful_widget(feed_list, area, &mut app.list_state);
    if let AppState::Popup(feed) = &app.state {
        let block = Block::bordered();
        let area = centered_rect(90, 90, area);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(block, area);
        let content_area = Rect {
            x: area.x + 2,
            y: area.y + 2,
            width: area.width - 4,
            height: area.height - 4,
        };

        // // In case of image ...
        // if let Feed::Item(item) = feed {
        //     if let Some(media) = item.extensions().get("media") {
        //         if let Some(content) = media.get("content") {
        //             for ext in content {
        //                 if let Some(image_url) = ext.attrs().get("url") {
        //                     let image_bytes = tokio::spawn(future)
        //                         reqwest::blocking::get(image_url).unwrap().bytes().unwrap();
        //                     let b = image::load_from_memory(&image_bytes).unwrap();
        //                     let mut picker = Picker::new((7, 9));
        //                     picker.guess_protocol();

        //                     let mut _aaa = picker.new_resize_protocol(b);
        //                 }
        //             }
        //         }
        //     }
        // }
        // let sf_image = StatefulImage::new(None);
        // frame.render_stateful_widget(sf_image, content_area, &mut aaa);

        frame.render_widget(Line::raw(feed.title()), content_area);
        if let Some(image) = &mut app.image {
            let sf_image = StatefulImage::new(None);
            frame.render_stateful_widget(
                sf_image,
                Rect {
                    x: content_area.x + 2,
                    y: content_area.y + 2,
                    width: content_area.width - 4,
                    height: 20,
                },
                image,
            );
        }

        frame.render_widget(
            Paragraph::new(feed.description()).wrap(Wrap { trim: false }),
            Rect {
                x: area.x + 2,
                y: content_area.y + 22,
                width: area.width - 4,
                height: area.height - 8,
            },
        );
    }
}

// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
