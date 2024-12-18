use crate::app::App;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType, List, Paragraph, Widget, Wrap},
    Frame,
};

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

    if let Some(popup) = &app.popup {
        if window_area.height > 10 {
            let popup_area = Rect {
                x: (window_area.width / 2) + window_area.width % 2,
                y: window_area.y,
                width: (window_area.width / 2),
                height: window_area.height,
            };
            //render_popup(popup, frame, popup_area);
            popup.render(popup_area, frame.buffer_mut());
        }
    }
}

fn render_instructions(frame: &mut Frame, window_area: Rect) {
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
