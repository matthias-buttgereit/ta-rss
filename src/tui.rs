use crate::app::App;
use crate::event::EventHandler;
use crate::render;
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::Backend;
use ratatui::Terminal;

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
