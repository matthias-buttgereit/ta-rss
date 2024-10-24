pub mod render;

use crate::{
    app::App,
    events::event::{Event, EventHandler},
};

pub async fn start_tui(mut app: App) -> anyhow::Result<()> {
    let mut tui = ratatui::init();
    let mut events = EventHandler::new(60);

    while app.running {
        match events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => EventHandler::handle_key_events(&mut app, key_event)?,
            Event::Mouse(mouse_event) => EventHandler::handle_mouse_events(&mut app, mouse_event)?,
            Event::Resize(_, _) => {}
            Event::Paste(text) => EventHandler::handle_paste_event(&mut app, &text)?,
        }

        tui.draw(|frame| render::render(&mut app, frame))?;
    }

    ratatui::restore();
    Ok(())
}
