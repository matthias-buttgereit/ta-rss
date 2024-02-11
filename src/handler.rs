use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }

        // Navigating within the list of feeds
        KeyCode::Up => {
            if !app.content_popup_open {
                app.select_previous();
            }
        }
        KeyCode::Down => {
            if !app.content_popup_open {
                app.select_next();
            }
        }
        // Popup handlers
        KeyCode::Esc => {
            if app.content_popup_open {
                app.content_popup_open = false;
            } else {
                app.quit();
            }
        }
        KeyCode::Char(' ') => {
            app.content_popup_open = !app.content_popup_open;
        }
        _ => {}
    }
    Ok(())
}
