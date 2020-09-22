use crate::redmine::TimeEntries;
use crate::track::Config;
use reqwest::blocking;

#[derive(Debug)]
pub struct Client {
    client: blocking::Client,
    config: Config,
}

impl Client {
    pub fn new(config: Config) -> Self {
        Client {
            client: blocking::Client::new(),
            config,
        }
    }

    pub fn get_time_entries(&self) -> anyhow::Result<TimeEntries> {
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();

        let key = self.config.key.clone();
        let user_id = self.config.user_id;
        let url = self.config.base_url.clone().join("time_entries.json")?;

        let result = self
            .client
            .get(url)
            .query(&[
                ("key", key),
                ("user_id", user_id.to_string()),
                ("spent_on", date),
            ])
            .send()?
            .json()?;

        Ok(result)
    }
}
