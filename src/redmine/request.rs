use reqwest::blocking;

use crate::redmine::{
    Activities, CustomFields, NewTimeEntries, NewTimeEntry, Project, Projects, TimeEntries,
};
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
        let user_id = self.config.user_id;

        self.get(
            "time_entries.json",
            vec![("user_id", user_id.to_string()), ("spent_on", date)],
        )
    }

    pub fn get_projects(&self) -> anyhow::Result<Projects> {
        let user_id = self.config.user_id;
        let query = vec![("user_id", user_id.to_string()), ("limit", 100.to_string())];

        self.get("projects.json", query)
    }

    pub fn get_activities(&self) -> anyhow::Result<Activities> {
        self.get("enumerations/time_entry_activities.json", vec![])
    }

    pub fn get_custom_fields(&self) -> anyhow::Result<CustomFields> {
        self.get("custom_fields.json", vec![])
    }

    pub fn create_time_entry(&self, entry: NewTimeEntry) -> anyhow::Result<()> {
        let key = self.config.key.clone();
        let url = self.config.base_url.clone().join("time_entries.json")?;
        let new_entry = NewTimeEntries { time_entry: entry };

        let result = self
            .client
            .post(url)
            .json(&new_entry)
            .query(&[("key", key)])
            .send()?
            .text()?;

        Ok(())
    }

    fn get<T>(&self, path: &str, query: Vec<(&str, String)>) -> anyhow::Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let key = self.config.key.clone();
        let url = self.config.base_url.clone().join(path)?;

        let mut x = query.clone();
        x.push(("key", key));

        let result: T = self.client.get(url).query(&x).send()?.json()?;

        Ok(result)
    }
}
