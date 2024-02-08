use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Clear, List},
    Frame,
};

use crate::app::App;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.size();

    let block = Block::bordered()
    .title("Ta-RSS")
    .border_style(Style::default())
    .border_type(BorderType::Rounded)
    .style(Style::default());

    let feed_list = List::new(app.feeds.clone()).highlight_style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Black)
            .bg(Color::Gray),
    ).block(block);

    frame.render_stateful_widget(feed_list, area, &mut app.list_state);
    if app.content_popup_open {
        let block = Block::bordered().title("Popup");
        let area = centered_rect(90, 90, area);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(block, area);
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
