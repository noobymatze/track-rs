pub mod request;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeEntries {
    pub time_entries: Vec<TimeEntry>,
    pub offset: i32,
    pub limit: i32,
    pub total_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Named {
    pub id: i32,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TimeEntry {
    pub id: i32,
    pub user: Named,
    pub project: Named,
    pub issue: Option<Named>,
    pub hours: f64,
    pub comments: Option<String>,
    pub spent_on: String,
    #[serde(default)]
    pub custom_fields: Vec<CustomValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTimeEntries {
    pub time_entry: NewTimeEntry,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTimeEntry {
    pub issue_id: Option<i32>,
    pub project_id: Option<i32>,
    pub spent_on: String,
    pub hours: f64,
    pub activity_id: i32,
    pub comments: String,
    pub custom_fields: Vec<CustomValue>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Projects {
    pub projects: Vec<Project>,
    pub offset: i32,
    pub limit: i32,
    pub total_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub identifier: String,
    pub parent: Option<Named>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Issues {
    pub issues: Vec<Issue>,
    pub offset: i32,
    pub limit: i32,
    pub total_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Results {
    pub results: Vec<Result>,
    pub offset: i32,
    pub limit: i32,
    pub total_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Result {
    pub id: i32,
    pub title: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Issue {
    pub id: i32,
    pub subject: String,
    pub project: Named,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Activities {
    #[serde(rename = "time_entry_activities")]
    pub activities: Vec<Activity>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Activity {
    pub id: i32,
    pub name: String,
    pub is_default: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomFields {
    pub custom_fields: Vec<CustomField>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct CustomField {
    pub id: i32,
    pub name: String,
    pub is_required: Option<bool>,
    pub field_format: String,
    pub customized_type: String,
}

impl CustomField {
    pub fn is_for_time_entry(&self) -> bool {
        self.customized_type == "time_entry"
    }

    pub fn is_required(&self) -> bool {
        self.is_required.unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CustomValue {
    pub id: i32,
    pub name: String,
    pub value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserResponse {
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub login: String,
    pub firstname: String,
    pub lastname: String,
    pub api_key: Option<String>,
}
