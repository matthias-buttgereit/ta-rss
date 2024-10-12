mod app;
mod event;
mod feed;
mod tui;

use app::{cli::Cli, App};
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    let cli = Cli::parse();
    match cli.command {
        Some(command) => app.handle_cli_command(command).await?,
        None => app.start_tui().await?,
    }

    Ok(())
}
