use chrono::NaiveDate;
use cli_table::format::Justify;
use cli_table::{Cell, Color, Row, Style, Table, TableStruct};
use std::collections::HashMap;

use crate::redmine::TimeEntry;

/// A [Report] represents the result of cumulating a [Vec] of [TimeEntry]s.
#[derive(Debug)]
pub struct Report {
    projects: HashMap<i32, String>,
    cumulative_hours: HashMap<(NaiveDate, i32), f64>,
    entries_per_day: HashMap<NaiveDate, Vec<TimeEntry>>,
    hours_per_project: HashMap<i32, f64>,
    hours_at: HashMap<NaiveDate, f64>,
}

impl Report {
    /// Creates a new [Report] from a vector of time entries.
    ///
    /// If the given time_entries are empty, the report will of course be empty too.
    pub fn from_entries(time_entries: &Vec<TimeEntry>) -> Self {
        let mut cumulative_hours = HashMap::new();
        let mut projects = HashMap::new();
        let mut hours_per_project = HashMap::new();
        let mut hours_at = HashMap::new();
        let mut entries_per_day = HashMap::new();

        for time_entry in time_entries {
            let spent_on = NaiveDate::parse_from_str(&time_entry.spent_on, "%Y-%m-%d").unwrap();

            let key = (spent_on, time_entry.project.id);
            *cumulative_hours.entry(key).or_insert(0.0) += time_entry.hours;

            entries_per_day
                .entry(spent_on)
                .or_insert(vec![])
                .push(time_entry.clone());

            *hours_per_project
                .entry(time_entry.project.id)
                .or_insert(0.0) += time_entry.hours;

            *hours_at.entry(spent_on).or_insert(0.0) += time_entry.hours;

            projects.entry(time_entry.project.id).or_insert(
                time_entry
                    .project
                    .name
                    .clone()
                    .expect("This should be set, since the ID exists."),
            );
        }

        Report {
            projects,
            entries_per_day,
            cumulative_hours,
            hours_per_project,
            hours_at,
        }
    }

    /// Returns a new [DailyReport] for the given [needle].
    ///
    /// If there are no time entries for the day, will return an
    /// empty [DailyReport].
    pub fn get_report_for_date(&self, needle: &NaiveDate) -> DailyReport {
        let total_hours = *self.hours_at.get(needle).unwrap_or(&0.0);

        let entries = self.entries_per_day.get(needle).unwrap_or(&vec![]).clone();

        DailyReport {
            total_hours,
            entries,
        }
    }
}

/// A [DailyReport] represents a special report for a single day.
#[derive(Debug)]
pub struct DailyReport {
    entries: Vec<TimeEntry>,
    total_hours: f64,
}

impl DailyReport {
    /// Converts a `DailyReport` into a `Table` for display.
    ///
    /// # Example
    ///
    /// Given the following `DailyReport`:
    ///
    /// ```
    /// let daily_report = DailyReport {
    ///     entries: vec![
    ///         TimeEntry {
    ///             id: 1,
    ///             user: Named { id: 1, name: "User A".to_owned() },
    ///             project: Named { id: 1, name: "Project A".to_owned() },
    ///             issue: Some(Named { id: 1234, name: "Issue A".to_owned() }),
    ///             hours: 9.0,
    ///             comments: Some("Worked on issue A".to_owned()),
    ///             spent_on: "2022-01-01".to_owned(),
    ///         },
    ///         TimeEntry {
    ///             id: 2,
    ///             user: Named { id: 1, name: "User A".to_owned() },
    ///             project: Named { id: 2, name: "Project B".to_owned() },
    ///             issue: None,
    ///             hours: 3.0,
    ///             comments: None,
    ///             spent_on: "2022-01-01".to_owned(),
    ///         },
    ///     ],
    ///     total_hours: 12.0,
    /// };
    /// ```
    ///
    /// Calling `to_table_struct` on the `DailyReport` will produce more or less the following `Table`:
    ///
    /// ```text
    /// +---------+----------+-------+---------------+
    /// | Project | Issue    | Hours | Comment       |
    /// +---------+----------+-------+---------------+
    /// | Project A | #1234  | 9.00 | Worked on issue A |
    /// | Project B |        | 3.00 |              |
    /// +---------+--------+-------+---------------+
    /// ```
    pub fn to_table_struct(&self) -> TableStruct {
        let headers = vec![
            "Project".cell().bold(true),
            "Issue".cell().bold(true),
            format!("Hours (∑ {:.2})", self.total_hours)
                .cell()
                .bold(true),
            "Comment".cell().bold(true),
        ];

        let mut rows = vec![];
        rows.push(headers.row());
        for entry in &self.entries {
            let cells = vec![
                entry.project.name.as_ref().unwrap_or(&"".into()).cell(),
                entry
                    .issue
                    .as_ref()
                    .map(|issue| format!("#{}", issue.id))
                    .unwrap_or("".into())
                    .cell()
                    .foreground_color(Some(Color::Cyan))
                    .justify(Justify::Right),
                format!("{:.2}", entry.hours).cell().justify(Justify::Right),
                format!("{}", entry.comments.as_ref().unwrap_or(&"".into()))
                    .cell()
                    .foreground_color(Some(Color::Rgb(230, 230, 230))),
            ];
            rows.push(cells.row());
        }

        rows.table()
    }
}

#[cfg(test)]
mod tests {
    use crate::redmine::Named;

    use super::*;

    #[test]
    fn test_calculate() {
        let day1 = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2022, 1, 3).unwrap();
        let day3 = NaiveDate::from_ymd_opt(2022, 1, 5).unwrap();

        let time_entries = vec![
            time_entry(1, (1, "John Doe"), (1, "Project A"), 4.0, &day1),
            time_entry(2, (2, "Jane Doe"), (2, "Project B"), 2.0, &day2),
            time_entry(3, (1, "John Doe"), (1, "Project A"), 5.0, &day3),
        ];

        let result = Report::from_entries(&time_entries);

        assert_eq!(result.projects.get(&1), Some(&"Project A".to_string()));
        assert_eq!(result.projects.get(&2), Some(&"Project B".to_string()));

        let expected_cumulative_hours = vec![((day1, 1), 4.0), ((day2, 2), 2.0), ((day3, 1), 5.0)];

        for (key, value) in expected_cumulative_hours {
            assert_eq!(result.cumulative_hours.get(&key), Some(&value));
        }
    }

    #[test]
    fn test_sum_of_hours_per_project_equals_sum_of_hours_per_weekday() {
        let day1 = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2022, 1, 3).unwrap();
        let day3 = NaiveDate::from_ymd_opt(2022, 1, 5).unwrap();

        let time_entries = vec![
            time_entry(1, (1, "John Doe"), (1, "Project A"), 4.0, &day1),
            time_entry(2, (2, "Jane Doe"), (2, "Project B"), 2.0, &day2),
            time_entry(3, (1, "John Doe"), (1, "Project A"), 5.0, &day3),
        ];

        let result = Report::from_entries(&time_entries);

        let sum_of_hours_per_project = result.hours_per_project.values().sum::<f64>();
        let sum_of_hours_per_weekday = result.hours_at.values().sum::<f64>();

        assert_eq!(sum_of_hours_per_project, sum_of_hours_per_weekday);
    }

    #[test]
    fn test_daily_report_calculation() {
        let day1 = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2022, 1, 3).unwrap();

        let day1_entries = vec![
            time_entry(1, (1, "John Doe"), (1, "Project A"), 4.0, &day1),
            time_entry(3, (1, "John Doe"), (1, "Project A"), 5.0, &day1),
        ];

        let mut time_entries = vec![
            time_entry(2, (2, "Jane Doe"), (2, "Project B"), 2.0, &day2),
            time_entry(4, (1, "John Doe"), (1, "Project A"), 5.0, &day2),
        ];
        time_entries.extend(day1_entries.clone());

        let report = Report::from_entries(&time_entries);
        let daily_report = report.get_report_for_date(&day1);

        let expected_total_hours = 9.0;
        assert_eq!(daily_report.total_hours, expected_total_hours);

        assert_eq!(daily_report.entries, day1_entries);
    }

    fn time_entry(
        id: i32,
        user: (i32, &str),
        project: (i32, &str),
        hours: f64,
        spent_on: &NaiveDate,
    ) -> TimeEntry {
        TimeEntry {
            id,
            user: Named {
                id: user.0,
                name: Some(user.1.to_string()),
            },
            project: Named {
                id: project.0,
                name: Some(project.1.to_string()),
            },
            issue: None,
            hours,
            comments: None,
            spent_on: spent_on.format("%Y-%m-%d").to_string(),
        }
    }
}
