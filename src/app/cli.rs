use clap::{Parser, Subcommand};

use crate::feed::entry::check_url;

use super::{config::Config, App};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new url to the list of feeds
    Add { url: String },
    /// Remove an url from the list of feeds
    Remove { url: String },
    /// List urls of all feeds
    List,
}

impl App {
    pub async fn handle_cli_command(&mut self, command: Commands) -> anyhow::Result<()> {
        match command {
            Commands::Add { url } => match self.add_feed(&url).await {
                Ok(title) => println!("Added feed: {title}"),
                Err(e) => println!("Failed to add feed: {e}"),
            },
            Commands::Remove { url } => match self.remove_feed(&url) {
                Ok(title) => println!("Removed feed: {title}"),
                Err(e) => println!("Failed to remove feed: {e}"),
            },
            Commands::List => self.print_feeds(),
        }
        Ok(())
    }

    fn remove_feed(&mut self, url: &str) -> anyhow::Result<String> {
        if self.feed_urls.is_empty() {
            return Err(anyhow::anyhow!(
                "No feeds added yet. Add one with 'ta-rss add <url>'"
            ));
        }

        if !self.feed_urls.contains(&url.to_string()) {
            return Err(anyhow::anyhow!("Failed to remove feed"));
        }

        self.feed_urls.retain(|x| !x.eq(&url.to_string()));
        let _ = Config::save(&self.feed_urls);
        Ok(format!("Removed feed: {url}"))
    }

    async fn add_feed(&mut self, url: &str) -> anyhow::Result<String> {
        let title = check_url(url).await?;

        if self.feed_urls.contains(&url.to_string()) {
            return Err(anyhow::anyhow!("Feed already added"));
        }
        self.feed_urls.push(url.to_string());
        Config::save(&self.feed_urls)?;

        Ok(title)
    }

    fn print_feeds(&self) {
        if self.feed_urls.is_empty() {
            println!("No feeds added yet. Add one with 'ta-rss add <url>'");
            return;
        }

        let mut feed_titles = String::new();
        for url in &self.feed_urls {
            feed_titles.push_str(&format!("{url}\n"));
        }
        print!("{feed_titles}");
    }
}
