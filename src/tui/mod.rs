pub mod render;

use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Backend, CrosstermBackend},
    Terminal,
};

use crate::{
    app::App,
    events::{
        event::{Event, EventHandler},
        handler::{handle_key_events, handle_paste_event},
    },
};

pub async fn start_tui(mut app: App) -> anyhow::Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(20);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(mouse_event) => {
                if mouse_event.kind == crossterm::event::MouseEventKind::ScrollDown {
                    app.scroll_down();
                }
                if mouse_event.kind == crossterm::event::MouseEventKind::ScrollUp {
                    app.scroll_up();
                }
            }
            Event::Resize(_, _) => {}
            Event::Paste(text) => handle_paste_event(&mut app, &text)?,
        }

        tui.draw(&mut app)?;
    }

    tui.exit()?;
    Ok(())
}

#[derive(Debug)]
pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    pub fn init(&mut self) -> anyhow::Result<()> {
        terminal::enable_raw_mode()?;
        ratatui::crossterm::execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        let panic_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> anyhow::Result<()> {
        self.terminal.draw(|frame| render::render(app, frame))?;
        Ok(())
    }

    fn reset() -> anyhow::Result<()> {
        terminal::disable_raw_mode()?;
        ratatui::crossterm::execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn exit(&mut self) -> anyhow::Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
