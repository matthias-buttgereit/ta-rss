use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::App;

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> anyhow::Result<()> {
    match key_event.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c' | 'C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::Char(' ') => {
            app.toggle_popup();
        }
        KeyCode::Char('o' | 'O') => {
            if let Some(entry) = &app.popup {
                let url = &entry.url;
                let _open_error = open::that_in_background(url);
            };
        }
        KeyCode::Esc => {
            if app.popup.is_some() {
                app.popup = None;
            } else {
                app.quit();
            }
        }
        KeyCode::Up => app.select_previous(),
        KeyCode::Down => app.select_next(),
        _ => {}
    }

    Ok(())
}

pub fn handle_paste_event(_app: &mut App, _text: &str) -> anyhow::Result<()> {
    todo!("Paste event not implemented yet. Depends on crossterm feature 'bracketed-paste'.");
}
