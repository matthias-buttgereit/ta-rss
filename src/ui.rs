use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{block::Title, Block, BorderType, Clear, List, Paragraph, Wrap},
    Frame,
};
use ratatui_image::StatefulImage;

use crate::{
    app::{App, AppState},
    feed::Feed,
};

pub fn render(app: &mut App, frame: &mut Frame) {
    render_list(app, frame, frame.size());

    if let AppState::Popup(feed) = &app.state {
        let popup_area = centered_rect(90, 90, frame.size());
        render_popup(app, frame, popup_area, feed);
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

fn render_popup(app: &App, frame: &mut Frame, area: Rect, feed: &Feed) {
    // Render popup window
    let block = Block::bordered()
        .title(feed.source_name())
        .title(Title::from(feed.pub_date_string()).alignment(Alignment::Right));
    let area = centered_rect(90, 90, area);
    // Clear the popup window
    frame.render_widget(Clear, area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(40),
            Constraint::Fill(1),
        ])
        .split(area);

    // Render feed title
    frame.render_widget(
        Paragraph::new(feed.title()).wrap(Wrap { trim: false }),
        chunks[0],
    );

    // Render feed image if there is one
    if let Some(image) = &app.current_feed_image {
        let sf_image = StatefulImage::new(None);
        let image = &mut image.clone();
        frame.render_stateful_widget(sf_image, centered_rect(90, 100, chunks[1]), image);
    }

    // Render feed description
    frame.render_widget(
        Paragraph::new(feed.description()).wrap(Wrap { trim: false }),
        chunks[2],
    );
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
