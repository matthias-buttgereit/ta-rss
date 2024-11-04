pub mod render;

use crate::{
    app::App,
    events::{Event, EventHandler},
};

pub async fn start(app: &mut App) -> anyhow::Result<()> {
    let mut tui = ratatui::init();
    let mut events = EventHandler::new(60);

    while app.running {
        tui.draw(|frame| render::render(app, frame))?;
        match events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => EventHandler::handle_key_events(app, key_event)?,
            Event::Mouse(mouse_event) => EventHandler::handle_mouse_events(app, mouse_event)?,
            Event::Resize(_, _) => {}
            Event::Paste(text) => EventHandler::handle_paste_event(app, &text)?,
        }
    }

    ratatui::restore();
    Ok(())
}
