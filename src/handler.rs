use crate::{
    app::{App, AppResult, AppState},
    feed::Feed,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui_image::picker::{Picker, ProtocolType};
use tokio::task::JoinHandle;

// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    handle_global_key_events(key_event, app)?;
    match app.state {
        AppState::List => handle_list_state(key_event, app)?,
        AppState::Popup(_) => handle_popup_state(key_event, app)?,
    };

    Ok(())
}

fn handle_global_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
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

fn handle_list_state(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Up => app.select_previous(),
        KeyCode::Down => app.select_next(),
        KeyCode::Esc => app.quit(),

        KeyCode::Char(' ') => {
            app.current_feed_image = None;
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
        _ => {}
    }

    Ok(())
}

fn handle_popup_state(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char(' ') => app.state = AppState::List,
        KeyCode::Esc => app.state = AppState::List,
        _ => {}
    }

    Ok(())
}
