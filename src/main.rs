use clap::Parser;
use ta_rss::app::{App, AppResult};
use ta_rss::start_tui;
use ta_rss::Arguments;

// Asynchronous main function
#[tokio::main]
async fn main() -> AppResult<()> {
    // Create a new instance of the application
    let mut app = App::new().await;

    // Parse command line arguments
    let args = Arguments::parse();

    // If the add argument is provided, add the feed and print a message
    if !args.add.is_empty() {
        match app.add_feed(&args.add).await {
            Ok(title) => println!("Added feed: {}", title),
            Err(e) => eprintln!("Failed to add feed: {}", e),
        }
    } else {
        // When no extra arguments are provided, start the TUI
        start_tui(app).await?;
    }
    Ok(())
}
