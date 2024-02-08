use clap::Parser;
use ta_rss::app::{App, AppResult};
use ta_rss::start_tui;
use ta_rss::Arguments;

// Asynchronous main function
#[tokio::main]
async fn main() -> AppResult<()> {
    // Parse command line arguments
    let args = Arguments::parse();

    // Create a new instance of the application
    let mut app = App::new().await;

    // If the add argument is provided, add the feed and print a message
    if !args.add.is_empty() {
        app.add_feed(&args.add).await;
        println!("Added feed: {}", args.add);
    } else {
        // Start the text-based user interface
        start_tui(app).await?;
    }
    Ok(())
}
