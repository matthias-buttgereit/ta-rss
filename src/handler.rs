use crate::app::{App, AppResult};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
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
        // Selection
        KeyCode::Up => {
            if !app.popup_enabled {
                app.select_previous();
            }
        }
        KeyCode::Down => {
            if !app.popup_enabled {
                app.select_next();
            }
        }
        // Popup handlers
        KeyCode::Esc => {
            if app.popup_enabled {
                app.popup_enabled = false;
            } else {
                app.quit();
            }
        }
        KeyCode::Char(' ') => {
            app.popup_enabled = !app.popup_enabled;
        }
        _ => {}
    }
    Ok(())
}
