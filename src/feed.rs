use reqwest::Client;
use tokio::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub enum Feed {
    Item(rss::Item),
    Entry(atom_syndication::Entry),
}

impl Feed {
    fn pub_date(&self) -> Option<atom_syndication::FixedDateTime> {
        match self {
            Feed::Item(item) => {
                Some(chrono::DateTime::parse_from_rfc2822(item.pub_date().unwrap()).unwrap())
            }
            Feed::Entry(entry) => Some(*entry.updated()),
        }
    }

    pub fn title(&self) -> String {
        match self {
            Feed::Item(item) => item.title().unwrap().to_string(),
            Feed::Entry(entry) => entry.title().to_string(),
        }
    }

    pub fn description(&self) -> String {
        match self {
            Feed::Item(item) => item.description().unwrap().to_string(),
            Feed::Entry(entry) => entry.summary().unwrap().to_string(),
        }
    }

    pub async fn fetch_and_parse(tx: Sender<Feed>, feed_urls: Vec<String>) {
        let client = Client::new();

        for url in feed_urls {
            let tx = tx.clone();
            let client = client.clone();
            tokio::spawn(async move {
                let result = client.get(url).send().await.unwrap().bytes().await.unwrap();
                if let Ok(feed) = rss::Channel::read_from(&result[..]) {
                    for item in feed.items {
                        tx.send(Feed::Item(item)).await.unwrap();
                    }
                } else if let Ok(channel) = atom_syndication::Feed::read_from(&result[..]) {
                    for entry in channel.entries {
                        tx.send(Feed::Entry(entry)).await.unwrap();
                    }
                }
            });
        }
    }
}

impl PartialOrd for Feed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.pub_date().partial_cmp(&other.pub_date())
    }
}

impl PartialEq for Feed {
    fn eq(&self, other: &Self) -> bool {
        self.pub_date() == other.pub_date()
    }
}
