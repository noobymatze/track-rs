use anyhow::anyhow;
use chrono::{DateTime, Local};
use reqwest::blocking;
use url::Url;

use crate::redmine::{
    Activities, CustomFields, Issues, NewTimeEntries, NewTimeEntry, Projects, Results, TimeEntries,
    User, UserResponse,
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

    pub fn get_issues(&self, issue_ids: &Vec<String>) -> anyhow::Result<Issues> {
        let issue_ids = issue_ids.join(",");
        let query = vec![
            ("issue_id", issue_ids),
            ("status_id", String::from("*")),
            ("limit", 100.to_string()),
        ];

        self.get("issues.json", query)
    }
    pub fn search_tickets(&self, query: String) -> anyhow::Result<Results> {
        let query = vec![
            ("q", [" ", &*query.to_string(), " "].join("")),
            ("limit", 100.to_string()),
            ("issues", 1.to_string()),
            ("titles_only", 1.to_string()),
        ];

        self.get("search.json", query)
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
        if !&self.config.custom_fields.is_empty() {
            return Ok(CustomFields {
                custom_fields: self.config.custom_fields.clone(),
            });
        }
        self.get("custom_fields.json", vec![])
    }

    pub fn create_time_entry(&self, entry: NewTimeEntry) -> anyhow::Result<()> {
        let key = &self.config.key;
        let url = &self.config.base_url.join("time_entries.json")?;
        let new_entry = NewTimeEntries { time_entry: entry };

        let response = self
            .client
            .post(url.clone())
            .json(&new_entry)
            .header("X-Redmine-API-Key", key)
            .send()?;

        if response.status().is_success() {
            return Ok(());
        }

        let status = &response.status();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| format!("{:?}: {:?}", name, value))
            .collect::<Vec<String>>()
            .join("\n");

        match &response.text() {
            Ok(value) => println!("{}", value),
            Err(err) => eprintln!("{:?}", err),
        }

        let msg = format!("{}\n\n{}", status, headers);

        Err(anyhow!(msg))
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
