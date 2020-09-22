pub mod request;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimeEntries {
    pub time_entries: Vec<TimeEntry>,
    pub offset: i32,
    pub limit: i32,
    pub total_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
