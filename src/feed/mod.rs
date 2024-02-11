use chrono::{DateTime, Utc};
use reqwest::Url;

pub mod atom;
pub mod rss;

pub trait Feed {
    fn get_items(&self) -> Vec<impl FeedItem>;
    fn get_title(&self) -> String;
    fn get_url(&self) -> Url;
    fn get_last_updated(&self) -> DateTime<Utc>;
}

pub trait FeedItem {
    fn get_title(&self) -> String;
    fn get_link(&self) -> Url;
    fn get_pub_date(&self) -> DateTime<Utc>;
}
