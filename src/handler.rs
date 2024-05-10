use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> anyhow::Result<()> {
    match key_event.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::Char(' ') => {
            todo!("Toggle popup");
        }
        KeyCode::Char('o') | KeyCode::Char('O') => {
            if let Some(entry) = app.popup {
                let url = &entry.url;
                let _open_error = open::that_in_background(url);
            };
        }
        KeyCode::Esc => {
            todo!("Exit popup or exit app.");
        }
        KeyCode::Up => app.select_previous(),
        KeyCode::Down => app.select_next(),
        _ => {}
    }

    Ok(())
}

pub fn _handle_paste_event(_app: &mut App, _text: String) -> anyhow::Result<()> {
    todo!("Paste event not implemented yet. Depends on crossterm feature 'bracketed-paste'.");
}
