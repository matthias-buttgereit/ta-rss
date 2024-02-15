use crate::{
    app::{App, AppResult, AppState},
    feed::Feed,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_image::picker::{Picker, ProtocolType};
use tokio::task::JoinHandle;

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
        KeyCode::Up => {
            if let AppState::List(_) = app.state {
                app.select_previous();
            }
        }
        KeyCode::Down => {
            if let AppState::List(_) = app.state {
                app.select_next();
            }
        }
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
                app.image = None;
                let selected = app.list_state.selected().unwrap_or_default();
                let selected_feed = app.feeds.get(selected).unwrap();
                let tx = app.image_sender.clone();

                let _handle: Option<JoinHandle<()>> = match selected_feed {
                    Feed::Item(item) => match item.extensions().get("media") {
                        Some(media) => match media.get("content") {
                            Some(content) => {
                                let mut counter = 0;
                                loop {
                                    let ext = &content[counter];
                                    if ext.attrs().contains_key("url") {
                                        let image_url = ext.attrs().get("url").unwrap().to_string();
                                        break Some(tokio::spawn(async move {
                                            let image_bytes = reqwest::get(image_url)
                                                .await
                                                .unwrap()
                                                .bytes()
                                                .await
                                                .unwrap();

                                            let b = image::load_from_memory(&image_bytes).unwrap();
                                            let mut picker = Picker::new((5, 10));
                                            picker.protocol_type = ProtocolType::Halfblocks;

                                            let image = picker.new_resize_protocol(b);
                                            tx.send(image).await.unwrap();
                                        }));
                                    }
                                    counter += 1;
                                }
                            }
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                };

                app.state = AppState::Popup(selected_feed.clone());
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

fn handle_esc_key(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    if let KeyCode::Esc = key_event.code {
        match app.state {
            AppState::List(_) => {
                app.quit();
            }
            AppState::Popup(_) => {
                app.state = AppState::List(vec![]);
            }
            _ => {}
        }
    }

    Ok(())
}
