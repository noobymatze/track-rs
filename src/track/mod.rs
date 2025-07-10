pub mod report;
mod ui;

use chrono::{Datelike, Duration};
use cli_table::{print_stdout, Cell, Color, Row, Style, Table};
use report::Report;
use serde::{Deserialize, Serialize};
use std::fs::File;

use std::io::{BufReader, BufWriter};

use crate::redmine::request::Client;
use crate::redmine::{CustomField, NewTimeEntry, User};
use crate::track::Error::{ApiKeyMissing, HomeDirNotFound};
use std::io;
use std::str::FromStr;
use thiserror::Error;
use url::Url;

/// Track a new value of the time.
pub fn track(client: &Client, yesterday: bool, id: Option<String>) -> Result<(), anyhow::Error> {
    let (project, issue) = match id {
        None => match ui::ask_for_issue() {
            None => {
                let projects = client.get_projects()?;
                let project = ui::select_project(projects);
                (project, None)
            }

            issue => (None, issue),
        }

        Some(issue) => (None, i32::from_str(&issue).ok())
    };

    let comment = ui::ask_for_comment();
    let hours = match ui::analyze_comments(comment.clone()) {
        Some((from, to)) => {
            let duration = to - from;
            let hours = duration.num_hours() as f64;
            let remaining_minutes = if hours <= 0.0 {
                duration.num_minutes()
            } else {
                duration.num_minutes() % (duration.num_hours() * 60)
            };

            let minutes: f64 = (remaining_minutes as f64 / 15.0) * 0.25;
            hours + minutes
        }
        None => ui::ask_for_hours(),
    };
    let activities = client.get_activities()?;
    let activity = ui::select_activity(activities);
    let custom_fields = client.get_custom_fields()?;

    let mut custom_values = vec![];
    for field in custom_fields.custom_fields {
        if field.is_for_time_entry() && field.is_required() {
            if let Some(value) = ui::ask_for_custom_field(field)? {
                custom_values.push(value)
            }
        }
    }

    let today = match yesterday {
        true => {
            println!("Creating TimeEntry for yesterday");
            chrono::Local::now() - Duration::days(1)
        }
        false => chrono::Local::now(),
    };

    let new_entry = NewTimeEntry {
        issue_id: issue,
        project_id: project.map(|p| p.id),
        hours,
        comments: comment,
        activity_id: activity.id,
        custom_fields: custom_values,
        spent_on: today.format("%Y-%m-%d").to_string(),
    };

    client.create_time_entry(new_entry)?;
    list(&client, false, false, false, None)?;

    Ok(())
}

/// Search for the given [`query`] using the given [`Config`] and
/// display the result to the console.
pub fn search(client: &Client, query: String, direct_track: bool) -> anyhow::Result<()> {
    let results = client.search_tickets(query)?;

    let headers = vec![
        "Id".cell().bold(true),
        "Title".cell().bold(true),
        "Url".cell().bold(true),
    ];
    let mut rows = vec![];
    rows.push(headers.row());
    for result in &results.results {
        let cells = vec![
            result.id.to_string().cell(),
            result.title.clone().cell(),
            result.url.clone().cell(),
        ];
        rows.push(cells.row())
    }

    let table = rows.table();

    print_stdout(
        table
            .dimmed(true)
            .foreground_color(Some(Color::Rgb(150, 150, 150))),
    )?;

    if direct_track && results.results.len() == 1 {
        track(client, false, Some(results.results.first().unwrap().id.to_string()))
    } else {
        Ok(())
    }

}

/// List the current.
pub fn list(client: &Client, with_issues: bool, previous: bool, week: bool, ignore_custom_field: Option<String>) -> anyhow::Result<()> {
    let day = match (previous, week) {
        (true, false) => chrono::Local::now() - Duration::days(1),
        (true, true) => chrono::Local::now() - Duration::days(7),
        _ => chrono::Local::now(),
    };

    let (from, to) = match week {
        true => {
            let weekday = day.weekday();
            let start = day - Duration::days(weekday.num_days_from_monday() as i64);
            let end = day + Duration::days(weekday.num_days_from_sunday() as i64);
            (start, Some(end))
        }
        false => (day, None),
    };

    let time_entries = client.get_time_entries(from, to)?;
    
    // Filter time entries based on ignore_custom_field if specified
    let filtered_entries = match ignore_custom_field {
        Some(field_name) => {
            time_entries.time_entries
                .into_iter()
                .filter(|entry| {
                    // Filter out entries where the custom field is set to a "truthy" value
                    // null values are treated as false
                    !entry.custom_fields.iter().any(|cf| {
                        cf.name == field_name && 
                        cf.value.as_ref().map_or(false, |v| v == "1" || v.to_lowercase() == "true")
                    })
                })
                .collect()
        }
        None => time_entries.time_entries,
    };
    
    match week {
        true => {
            let issue_ids = &filtered_entries
                .iter()
                .filter_map(|t| t.issue.as_ref().map(|i| i.id))
                .map(|id| id.to_string())
                .collect::<Vec<String>>();
            let issues = client.get_issues(&issue_ids)?;
            let report = Report::from_entries(&filtered_entries, &issues.issues);

            let table =
                report.to_table_struct(&(from + Duration::days(1)).date_naive(), with_issues);
            print_stdout(
                table
                    .dimmed(true)
                    .foreground_color(Some(Color::Rgb(150, 150, 150))),
            )?;
            Ok(())
        }
        false => {
            let report = Report::from_entries(&filtered_entries, &vec![]);
            let daily_report = report.get_report_for_date(&from.date_naive());
            let table = daily_report.to_table_struct();
            print_stdout(
                table
                    .dimmed(true)
                    .foreground_color(Some(Color::Rgb(150, 150, 150))),
            )?;
            Ok(())
        }
    }
}

/// A `Config` defines all parameters necessary, to connect to a Redmine server.
///
/// It is stored in the users home directory as a .track file.
#[derive(Serialize, Deserialize, Debug, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub key: String,
    pub base_url: Url,
    pub login: String,
    pub user_id: i32,
    #[serde(default)]
    pub custom_fields: Vec<CustomField>,
}

/// This type represents any error, that can happen while loading or storing
/// a `Config`.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Your home directory could not be found.")]
    HomeDirNotFound,
    #[error("The configuration could not be read or written.")]
    Io(#[from] io::Error),
    #[error("The configuration could not be (de)/serialized.")]
    Json(#[from] serde_json::Error),
    #[error("The API key is missing, please create one in your user settings.")]
    ApiKeyMissing,
}

impl Config {
    /// Returns a new `Config` based on the `base_url` and `user`.
    ///
    /// ## Example
    ///
    /// ```
    /// let config = Config::new("https://myredmine.com", user);
    /// assert(config.user_id, 1);
    /// ```
    pub fn new(base_url: Url, user: User) -> Result<Self, Error> {
        let config = Config {
            key: user.api_key.ok_or(ApiKeyMissing)?,
            base_url,
            login: user.login,
            user_id: user.id,
            custom_fields: vec![],
        };

        Ok(config)
    }

    /// Load the configuration from `~/.track`.
    ///
    /// ## Example
    ///
    /// ```
    /// let result = Config::load();
    /// ```
    ///
    pub fn load() -> Result<Option<Self>, Error> {
        let home_dir = dirs::home_dir().ok_or(Error::HomeDirNotFound)?;
        let track_file = home_dir.join(".track");

        if !track_file.exists() {
            return Ok(None);
        }

        let file = File::open(track_file)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(Some(config))
    }

    /// Store this configuration in `~/.track`.
    ///
    /// ## Example
    ///
    /// ```
    /// let config = Config::new("https://myredmine.com", user);
    /// config.store();
    /// ```
    ///
    pub fn store(&self) -> Result<(), Error> {
        let home_dir = dirs::home_dir().ok_or(HomeDirNotFound)?;
        let track_file = home_dir.join(".track");
        let file = File::create(track_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;

        Ok(())
    }
}
