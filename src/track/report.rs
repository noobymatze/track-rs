use chrono::{Datelike, NaiveDate, Weekday};
use std::collections::HashMap;

use crate::redmine::TimeEntry;

/// A struct representing the result of cumulating [TimeEntry] by weekday and project.
pub struct Report {
    pub project_names: HashMap<i32, String>,
    pub cumulative_hours: HashMap<(Weekday, i32), f64>,
}

/// Creates a new [Report] from a vector of time entries.
///
/// # Arguments
///
/// * `time_entries` - A `Vec<TimeEntry>` containing the time entries to cumulate.
///
/// # Returns
///
/// A new `Report` struct containing the cumulated time entries.
///
pub fn calculate(time_entries: &Vec<TimeEntry>) -> Report {
    let mut cumulative_hours = HashMap::new();
    let mut project_names = HashMap::new();

    for time_entry in time_entries {
        let spent_on = NaiveDate::parse_from_str(&time_entry.spent_on, "%Y-%m-%d").unwrap();
        let weekday = spent_on.weekday();

        let key = (weekday, time_entry.project.id);
        *cumulative_hours.entry(key).or_insert(0.0) += time_entry.hours;

        project_names.entry(time_entry.project.id).or_insert(
            time_entry
                .project
                .name
                .clone()
                .expect("This should be set, since the ID exists."),
        );
    }

    Report {
        project_names,
        cumulative_hours,
    }
}

#[cfg(test)]
mod tests {
    use crate::redmine::Named;

    use super::*;

    #[test]
    fn test_calculate() {
        let time_entries = vec![
            TimeEntry {
                id: 1,
                user: Named {
                    id: 1,
                    name: Some("John Doe".to_string()),
                },
                project: Named {
                    id: 1,
                    name: Some("Project A".to_string()),
                },
                issue: None,
                hours: 4.0,
                comments: None,
                spent_on: "2022-01-01".to_string(),
            },
            TimeEntry {
                id: 2,
                user: Named {
                    id: 2,
                    name: Some("Jane Doe".to_string()),
                },
                project: Named {
                    id: 2,
                    name: Some("Project B".to_string()),
                },
                issue: None,
                hours: 2.0,
                comments: None,
                spent_on: "2022-01-03".to_string(),
            },
            TimeEntry {
                id: 3,
                user: Named {
                    id: 1,
                    name: Some("John Doe".to_string()),
                },
                project: Named {
                    id: 1,
                    name: Some("Project A".to_string()),
                },
                issue: None,
                hours: 5.0,
                comments: None,
                spent_on: "2022-01-05".to_string(),
            },
        ];

        let result = calculate(&time_entries);

        assert_eq!(result.project_names.get(&1), Some(&"Project A".to_string()));
        assert_eq!(result.project_names.get(&2), Some(&"Project B".to_string()));

        let expected_cumulative_hours = vec![
            ((Weekday::Sat, 1), 4.0),
            ((Weekday::Mon, 2), 2.0),
            ((Weekday::Wed, 1), 5.0),
        ];

        for (key, value) in expected_cumulative_hours {
            assert_eq!(result.cumulative_hours.get(&key), Some(&value));
        }
    }
}
