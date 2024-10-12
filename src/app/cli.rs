use clap::{Parser, Subcommand};

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
