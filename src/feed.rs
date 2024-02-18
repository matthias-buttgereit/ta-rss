use atom_syndication::Text;
use chrono::Datelike;

#[derive(Clone, Debug)]
pub enum Feed {
    Item(rss::Item),
    Entry(atom_syndication::Entry),
}

impl Feed {
    fn pub_date(&self) -> Option<atom_syndication::FixedDateTime> {
        match self {
            Feed::Item(item) => Some(
                chrono::DateTime::parse_from_rfc2822(item.pub_date().unwrap_or_default())
                    .unwrap_or_default(),
            ),
            Feed::Entry(entry) => Some(*entry.updated()),
        }
    }

    pub fn source_name(&self) -> String {
        match self {
            Feed::Item(item) => match item.source() {
                Some(source) => source.title().unwrap_or(source.url()).to_string(),
                None => "a".to_string(),
            },
            Feed::Entry(entry) => match entry.source() {
                Some(source) => source.title().value.to_string(),
                None => "b".to_string(),
            },
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
            Feed::Item(item) => item.description().unwrap_or(" ").to_string(),
            Feed::Entry(entry) => entry.summary().unwrap_or(&Text::plain(" ")).to_string(),
        }
    }

    pub fn pub_date_string(&self) -> String {
        let now = chrono::offset::Local::now();
        let time = self.pub_date().unwrap_or_default();

        if time.year() == now.year() && time.month() == now.month() && time.day() == now.day() {
            time.format("%H:%M").to_string()
        } else {
            time.format("%Y-%m-%d %H:%M").to_string()
        }
    }
}

impl PartialOrd for Feed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Feed {
    fn eq(&self, other: &Self) -> bool {
        self.pub_date() == other.pub_date() && self.source_name() == other.source_name()
    }
}

impl Eq for Feed {}

impl Ord for Feed {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.pub_date().cmp(&self.pub_date())
    }
}
