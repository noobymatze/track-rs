use std::collections::HashMap;

use chrono::{Datelike, Days, NaiveDate};
use cli_table::format::Justify;
use cli_table::{Cell, Color, Row, Style, Table, TableStruct};

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

    pub fn get_or_zero(&self, day: &NaiveDate, project_id: i32) -> f64 {
        *self
            .cumulative_hours
            .get(&(*day, project_id))
            .unwrap_or(&0.0)
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

    pub fn to_table_struct(&self, needle: &NaiveDate) -> TableStruct {
        let monday = needle
            .monday_of_week()
            .unwrap_or_else(|| panic!("The monday of {needle} should exist."));
        let sunday = needle
            .sunday_of_week()
            .unwrap_or_else(|| panic!("The sunday of {needle} should exist."));

        let days: Vec<NaiveDate> = monday
            .iter_days()
            .take_while(|day| Some(*day) != sunday.succ_opt())
            .collect();

        let fg = Some(Color::Rgb(220, 220, 220));

        let mut headers = vec!["".cell().bold(true), "∑".cell().justify(Justify::Right)];
        for day in &days {
            headers.push(day.weekday().to_string().cell().foreground_color(fg));
        }

        let mut rows = vec![];
        rows.push(headers.row());
        let mut total_hours: f64 = 0.0;
        let mut projects = self
            .projects
            .iter()
            .map(|(a, b)| (*a, b.clone()))
            .collect::<Vec<(i32, String)>>();
        projects.sort_by(|(_, a), (_, b)| a.cmp(b));
        for (project_id, project) in projects {
            let mut cols = vec![];
            cols.push(project.cell().foreground_color(fg));
            let project_hours = self.hours_per_project.get(&project_id).unwrap_or(&0.0);
            total_hours += project_hours;
            cols.push(
                project_hours
                    .fmt_zero_empty()
                    .cell()
                    .justify(Justify::Right)
                    .foreground_color(fg),
            );
            for day in &days {
                let hours = self.get_or_zero(day, project_id);
                cols.push(
                    hours
                        .fmt_zero_empty()
                        .cell()
                        .foreground_color(fg)
                        .justify(Justify::Right),
                )
            }
            rows.push(cols.row());
        }
        let mut last_row = vec!["∑".cell()];
        last_row.push(
            total_hours
                .cell()
                .justify(Justify::Right)
                .foreground_color(Some(Color::Cyan)),
        );
        for day in &days {
            let hours_at_day = *self.hours_at.get(day).unwrap_or(&0.0);
            let color = match hours_at_day {
                _ if hours_at_day <= 8.0 => Color::Green,
                _ if hours_at_day <= 10.0 => Color::Yellow,
                _ => Color::Red,
            };
            last_row.push(
                hours_at_day
                    .fmt_zero_empty()
                    .cell()
                    .justify(Justify::Right)
                    .foreground_color(Some(color)),
            );
        }

        rows.push(last_row.row());
        rows.table()
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
        let mut actual_entries = self.entries.clone();
        actual_entries.reverse();
        for entry in &actual_entries {
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
                entry
                    .comments
                    .as_ref()
                    .unwrap_or(&"".into())
                    .to_string()
                    .cell()
                    .foreground_color(Some(Color::Rgb(230, 230, 230))),
            ];
            rows.push(cells.row());
        }

        rows.table()
    }
}

trait NaiveDateExt {
    fn monday_of_week(&self) -> Option<NaiveDate>;

    fn sunday_of_week(&self) -> Option<NaiveDate>;
}

impl NaiveDateExt for NaiveDate {
    fn monday_of_week(&self) -> Option<NaiveDate> {
        let diff = self.weekday().num_days_from_monday();
        self.checked_sub_days(Days::new(diff as u64))
    }

    fn sunday_of_week(&self) -> Option<NaiveDate> {
        let diff = self.weekday().num_days_from_sunday();
        self.checked_add_days(Days::new((7 - diff) as u64))
    }
}

trait DisplayExt {
    fn fmt_zero_empty(self) -> String;
}

impl DisplayExt for f64 {
    fn fmt_zero_empty(self) -> String {
        if self == 0.0 {
            "".into()
        } else {
            self.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::redmine::Named;

    use super::*;

    #[test]
    fn test_days_till_sunday() {
        let monday = NaiveDate::from_ymd_opt(2023, 2, 16).unwrap();
        let funday = NaiveDate::from_ymd_opt(2023, 2, 19).unwrap();
        assert_eq!(monday.sunday_of_week(), Some(funday));
    }

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
