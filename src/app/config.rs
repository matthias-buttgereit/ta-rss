use serde::{Deserialize, Serialize};

use super::CONFIG_FILE_NAME;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    feed_urls: Vec<String>,
}

impl Config {
    pub fn load() -> anyhow::Result<Vec<String>> {
        let config_as_json = std::fs::read_to_string(CONFIG_FILE_NAME)?;
        let config: Config = serde_json::from_str(&config_as_json)?;
        Ok(config.feed_urls)
    }

    pub fn save(urls: &[String]) -> anyhow::Result<()> {
        let current_config = Config {
            feed_urls: urls.to_vec(),
        };
        let config_as_json = serde_json::to_string_pretty(&current_config)?;
        std::fs::write(CONFIG_FILE_NAME, config_as_json)?;
        Ok(())
    }
}
