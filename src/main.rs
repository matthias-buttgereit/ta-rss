use clap::Parser;
use ta_rss::app::App;
use ta_rss::Commands;
use ta_rss::{start_tui, Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut app = App::new().await;

    match cli.command {
        Some(Commands::Add { url }) => match app.add_feed(&url).await {
            Ok(title) => println!("Added feed: {}", title),
            Err(e) => eprintln!("Failed to add feed: {}", e),
        },
        Some(Commands::Remove { url }) => match app.remove_feed(&url) {
            Ok(title) => println!("Removed feed: {}", title),
            Err(e) => eprintln!("Failed to remove feed: {}", e),
        },
        Some(Commands::List) => app.print_feeds(),
        None => {
            start_tui(app).await?;
        }
    }

    Ok(())
}
