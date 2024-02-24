use crate::{
    app::{App, AppState},
    feed::Feed,
};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{block::Title, Block, BorderType, Clear, List, Paragraph, Wrap},
    Frame,
};
use ratatui_image::StatefulImage;

pub fn render(app: &mut App, frame: &mut Frame) {
    let window_area = frame.size();
    render_list(
        app,
        frame,
        Rect {
            height: window_area.height - 1,
            ..window_area
        },
    );
    if app.feeds.is_empty() {
        render_instructions(frame, window_area);
    }

    render_keybindings(
        app,
        frame,
        Rect {
            height: 1,
            y: window_area.height - 1,
            ..window_area
        },
    );

    if let AppState::Popup(feed) = &app.app_state {
        let popup_area = Rect {
            x: (window_area.width / 2),
            y: window_area.y + 1,
            width: (window_area.width / 2),
            height: window_area.height - 3,
        };
        render_popup(app, frame, popup_area, feed);
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

fn render_popup(app: &App, frame: &mut Frame, area: Rect, feed: &Feed) {
    // Extract and convert relevant data
    let date = feed.pub_date_string();
    let source = {
        let mut source = feed.source_name();
        let source_len = area.width as usize - (date.len() + 4);
        source.truncate(source_len);
        source
    };
    let title = Paragraph::new(feed.title()).wrap(Wrap { trim: true });
    let description = Paragraph::new(feed.description()).wrap(Wrap { trim: true });
    let image = &app.current_feed_image;

    // Set-up layout
    let title_area = Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: area.width - 4,
        height: 2,
    };
    let mut image_area = Rect::default();
    let mut y_coordinate = title_area.y + 3;
    if image.is_some() {
        image_area = Rect {
            x: area.x + 2,
            y: y_coordinate,
            width: area.width - 4,
            height: (area.width - 4) / 4, // clamp height to not overflow in short terminals
        };
    }
    let description_area = Rect {
        x: area.x + 2,
        y: y_coordinate,
        width: area.width - 4,
        height: 9,
    };

    let popup_height = title_area.height + description_area.height + image_area.height + 6;
    let popup_area = Rect {
        height: popup_height,
        ..area
    };

    // Render everything
    let block = Block::bordered()
        .title(source)
        .title(Title::from(date).alignment(Alignment::Right));
    // Clear the popup window
    frame.render_widget(Clear, popup_area);
    frame.render_widget(block, popup_area);
    // Render feed title
    frame.render_widget(title, title_area);
    if let Some(image) = image {
        let sf_image = StatefulImage::new(None);
        let image = &mut image.clone();
        frame.render_stateful_widget(sf_image, image_area, image);

        y_coordinate = image_area.y + image_area.height + 1;
    }

    // Render feed description
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
    )
}

fn render_list(app: &mut App, frame: &mut Frame, area: Rect) {
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
}
