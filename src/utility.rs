use std::str::FromStr;

use atom_syndication::Feed;
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
        dbg!("Parsing as RSS feed");
        Ok(FeedType::Rss(rss_channel))
    } else if let Ok(atom_feed) = Feed::from_str(&feed_string) {
        dbg!("Parsing as Atom feed");
        Ok(FeedType::Feed(atom_feed))
    } else {
        Err("Failed to parse as RSS or Atom feed".to_string())
    }
}

pub async fn load_feed_titles() -> Vec<String> {
    let feeds = vec![
        fetch_and_parse_feed("https://alternativeto.net/news/feed/")
            .await
            .unwrap(),
        fetch_and_parse_feed("https://rss.golem.de/rss.php?feed=ATOM1.0")
            .await
            .unwrap(),
    ];

    let mut titles = vec![];

    for feed in feeds {
        match feed {
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
