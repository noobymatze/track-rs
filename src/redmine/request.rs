use reqwest::blocking;

use crate::redmine::{
    Activities, CustomFields, NewTimeEntries, NewTimeEntry, Projects, TimeEntries, User,
    UserResponse,
};
use crate::track::Config;
use chrono::{DateTime, Local};
use url::Url;

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

    pub fn get_time_entries(
        &self,
        start: DateTime<Local>,
        end: Option<DateTime<Local>>,
    ) -> anyhow::Result<TimeEntries> {
        let date = start.format("%Y-%m-%d").to_string();
        let user_id = self.config.user_id;
        let mut query = vec![("user_id", user_id.to_string()), ("limit", 100.to_string())];
        match end {
            None => {
                query.push(("spent_on", date));
            }
            Some(end) => {
                let s = end.format("%Y-%m-%d").to_string();
                query.push(("from", date));
                query.push(("to", s));
            }
        };

        self.get("time_entries.json", query)
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
        let key = &self.config.key;
        let url = &self.config.base_url.join("time_entries.json")?;
        let new_entry = NewTimeEntries { time_entry: entry };

        let _result = self
            .client
            .post(url.clone())
            .json(&new_entry)
            .header("X-Redmine-API-Key", key)
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

        let result = self
            .client
            .get(url)
            .header("X-Redmine-API-Key", key)
            .query(&query)
            .send()?
            .json()?;

        Ok(result)
    }
}

pub fn login(
    client: blocking::Client,
    base_url: Url,
    user: String,
    password: String,
) -> anyhow::Result<User> {
    let url = base_url.join("users/current.json")?;
    let result: UserResponse = client
        .get(url)
        .basic_auth(user, Some(password))
        .send()?
        .json()?;

    Ok(result.user)
}
