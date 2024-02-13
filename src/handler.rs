use crate::app::{App, AppResult, AppState, _Feed};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Char(_) => {
            handle_char_keys(key_event, app)?;
        }

        // Handle Arrow keys
        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
            handle_arrow_keys(key_event, app)?;
        }

        // Popup handlers
        KeyCode::Esc => {
            handle_esc_key(key_event, app)?;
        }
        _ => {}
    }
    Ok(())
}

fn handle_arrow_keys(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Up => match app.state {
            AppState::List(_) => {
                app.select_previous();
            }
            _ => {
                todo!()
            }
        },
        KeyCode::Down => match app.state {
            AppState::List(_) => {
                app.select_next();
            }
            _ => {
                todo!()
            }
        },
        _ => {}
    }

    Ok(())
}

fn handle_char_keys(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
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
        KeyCode::Char(' ') => match app.state {
            AppState::List(_) => {
                app.state = AppState::Popup(_Feed {});
            }
            AppState::Popup(_) => {
                app.state = AppState::List(vec![]);
            }
            _ => {}
        },
    }
    Ok(())
}

fn handle_esc_key(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Esc => match app.state {
            AppState::List(_) => {
                app.quit();
            }
            AppState::Popup(_) => {
                app.state = AppState::List(vec![]);
            }
            _ => {}
        },
        _ => {}
    }

    Ok(())
}
