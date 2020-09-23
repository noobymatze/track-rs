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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeEntry {
    pub id: i32,
    pub user: Named,
    pub project: Named,
    pub issue: Option<Named>,
    pub hours: f64,
    pub comments: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomValue {
    id: i32,
    value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NewTimeEntry {
    pub issue_id: i32,
    pub project_id: i32,
    pub spent_on: String,
    pub hours: f64,
    pub activity: String,
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
