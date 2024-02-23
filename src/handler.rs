use crate::app::{App, AppResult, AppState};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    global_key_events(key_event, app)?;
    match app.app_state {
        AppState::List => list_state(key_event, app)?,
        AppState::Popup(_) => popup_state(key_event, app)?,
        _ => {}
    };

    Ok(())
}

fn global_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        _ => {}
    };
    Ok(())
}

fn list_state(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Up => app.select_previous(),
        KeyCode::Down => app.select_next(),
        KeyCode::Esc => app.quit(),

        KeyCode::Char(' ') => {
            if let Some(selected) = app.list_state.selected() {
                let selected_feed = app.feeds.get(selected).unwrap();
                app.app_state = AppState::Popup(selected_feed.clone());
            }
        }
        _ => {}
    }

    Ok(())
}

fn popup_state(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('o') | KeyCode::Char('O') => {
            if let AppState::Popup(feed) = &app.app_state {
                let url = feed.url();
                let _open_error = open::that_in_background(url);
            };
        }
        KeyCode::Char(' ') => app.app_state = AppState::List,
        KeyCode::Esc => app.app_state = AppState::List,
        KeyCode::Up => app.select_previous(),
        KeyCode::Down => app.select_next(),
        _ => {}
    }

    Ok(())
}

pub fn _handle_paste_event(app: &mut App, text: String) -> AppResult<()> {
    app.app_state = AppState::PastedLink(text);
    todo!("Paste event not implemented yet. Depends on crossterm feature 'bracketed-paste'.");
}
