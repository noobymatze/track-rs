use reqwest::blocking;

use crate::redmine::{Activities, Project, Projects, TimeEntries};
use crate::track::Config;

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

    pub fn get_projects(&self) -> anyhow::Result<Projects> {
        let key = self.config.key.clone();
        let user_id = self.config.user_id;
        let url = self.config.base_url.clone().join("projects.json")?;

        let result = self
            .client
            .get(url)
            .query(&[
                ("key", key),
                ("user_id", user_id.to_string()),
                ("limit", 100.to_string()),
            ])
            .send()?
            .json()?;

        Ok(result)
    }

    pub fn get_activities(&self) -> anyhow::Result<Activities> {
        let key = self.config.key.clone();
        let url = self.config.base_url.clone()
            .join("enumerations")?
            .join("time_entry_activities.json")?;

        let result = self.client.get(url).query(&[("key", key)]).send()?.json()?;

        Ok(result)
    }
}
