use std::str::FromStr;

use atom_syndication::Feed;
use futures::future::join_all;
use rss::Channel;

#[derive(Debug)]
pub enum FeedType {
    Rss(Channel),
    Feed(Feed),
}

async fn fetch_feed_string(url: &str) -> anyhow::Result<String> {
    let response = reqwest::get(url).await?;
    Ok(response.text().await?)
}

pub async fn fetch_and_parse_feed(url: &str) -> Result<FeedType, String> {
    let feed_string = fetch_feed_string(url).await.expect("Failed to fetch feed");

    // Try parsing as RSS
    if let Ok(rss_channel) = Channel::from_str(&feed_string) {
        Ok(FeedType::Rss(rss_channel))
    } else if let Ok(atom_feed) = Feed::from_str(&feed_string) {
        Ok(FeedType::Feed(atom_feed))
    } else {
        Err("Failed to parse as RSS or Atom feed".to_string())
    }
}

pub async fn load_feed_titles(urls: &Vec<String>) -> Vec<String> {
    let feeds = join_all(urls.iter().map(|url| fetch_and_parse_feed(url))).await;

    let mut titles = vec![];

    for feed in feeds {
        match feed.unwrap() {
            FeedType::Rss(channel) => channel.items().iter().for_each(|item| {
                titles.push(item.title().unwrap().to_string());
            }),
            FeedType::Feed(feed) => feed.entries().iter().for_each(|entry| {
                titles.push(entry.title().to_string());
            }),
        }
    }

    titles
}
