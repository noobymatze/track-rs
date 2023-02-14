use std::collections::HashMap;

use chrono::{Datelike, NaiveDate, Weekday};
use cli_table::{format::Justify, Cell, CellStruct, Row, RowStruct, Style, Table, TableStruct};

use crate::redmine::{TimeEntries, TimeEntry};

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

pub fn view_weekday_working_hours(time_entries: TimeEntries) -> Result<TableStruct, anyhow::Error> {
    let mut map = HashMap::new();
    for t in time_entries.time_entries {
        let date = chrono::NaiveDate::parse_from_str(&*t.spent_on, "%Y-%m-%d")?;
        let v = map.entry(date.weekday()).or_insert(0.0);
        *v += t.hours;
    }

    let headers = vec![
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    let mut all_headers: Vec<CellStruct> = headers
        .iter()
        .map(|w| w.to_string().cell().bold(true).justify(Justify::Right))
        .collect();
    all_headers.push("∑".cell().bold(true).justify(Justify::Right));

    let mut cells: Vec<CellStruct> = headers
        .iter()
        .map(|w| map.get(w).unwrap_or(&0.0))
        .map(|v| v.to_string().cell().justify(Justify::Right))
        .collect();

    let sum: f64 = map.values().sum();

    cells.push(sum.to_string().cell().justify(Justify::Right));

    let rows = vec![all_headers.row(), cells.row()];

    Ok(rows.table())
}

pub fn print_table(time_entries: &Vec<TimeEntry>) -> Result<TableStruct, anyhow::Error> {
    let mut grouped_entries = HashMap::new();
    let weekdays = vec![
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    let mut projects = HashMap::new();

    for time_entry in time_entries {
        let spent_on = NaiveDate::parse_from_str(&time_entry.spent_on, "%Y-%m-%d").unwrap();
        let weekday = spent_on.weekday();

        grouped_entries
            .entry((weekday, time_entry.project.id))
            .or_insert_with(|| vec![])
            .push(time_entry);

        projects
            .entry(time_entry.project.id)
            .or_insert(time_entry.project.name.clone());
    }

    let mut table = vec![];
    let mut header = vec!["Project".to_string().cell()];
    header.extend(
        weekdays
            .iter()
            .map(|weekday| weekday.to_string().cell())
            .collect::<Vec<CellStruct>>(),
    );
    header.push("∑".to_string().cell().justify(Justify::Right));
    table.push(header.row());

    let mut total = vec![0f64; weekdays.len() + 1];

    for (project_id, project_name) in &projects {
        let mut row = vec![project_name.as_ref().unwrap_or(&"".into()).cell()];
        row.extend(
            weekdays
                .iter()
                .map(|weekday| {
                    let cumulative_hours = grouped_entries
                        .get(&(*weekday, *project_id))
                        .map_or(0f64, |entries| {
                            entries.iter().map(|entry| entry.hours).sum()
                        });
                    total[weekdays.iter().position(|w| w == weekday).unwrap()] += cumulative_hours;
                    cumulative_hours
                        .fmt_zero_empty()
                        .cell()
                        .justify(Justify::Right)
                })
                .collect::<Vec<_>>(),
        );
        row.push(
            weekdays
                .iter()
                .map(|weekday| {
                    let cumulative_hours = grouped_entries
                        .get(&(*weekday, *project_id))
                        .map_or(0f64, |entries| {
                            entries.iter().map(|entry| entry.hours).sum()
                        });
                    cumulative_hours
                })
                .sum::<f64>()
                .fmt_zero_empty()
                .cell()
                .justify(Justify::Right),
        );
        total[weekdays.len()] += weekdays
            .iter()
            .map(|weekday| {
                grouped_entries
                    .get(&(*weekday, *project_id))
                    .map_or(0f64, |entries| {
                        entries.iter().map(|entry| entry.hours).sum()
                    })
            })
            .sum::<f64>();
        table.push(row.row());
    }

    let mut row = vec!["∑".to_string().cell()];
    row.extend(
        weekdays
            .iter()
            .map(|weekday| {
                total[weekdays.iter().position(|w| w == weekday).unwrap()]
                    .to_string()
                    .cell()
                    .justify(Justify::Right)
            })
            .collect::<Vec<_>>(),
    );
    row.push(
        total[weekdays.len()]
            .to_string()
            .cell()
            .justify(Justify::Right),
    );
    table.push(row.row());

    Ok(table.table())
}

pub fn view_hours_per_project(time_entries: TimeEntries) -> Result<TableStruct, anyhow::Error> {
    let mut grouped_entries = HashMap::new();
    for time_entry in &time_entries.time_entries {
        grouped_entries
            .entry(time_entry.project.id)
            .or_insert_with(|| vec![])
            .push(time_entry);
    }

    let total_hours: f64 = time_entries
        .time_entries
        .iter()
        .map(|entry| entry.hours)
        .sum();

    let mut rows = vec![];

    rows.push(
        vec![
            "Project".cell().bold(true),
            format!("Hours (∑ {})", total_hours).cell().bold(true),
        ]
        .row(),
    );

    for (_, project_entries) in &grouped_entries {
        let project_name = &project_entries[0].project.name;
        let cumulative_hours: f64 = project_entries.iter().map(|entry| entry.hours).sum();

        rows.push(
            vec![
                project_name.as_ref().unwrap_or(&"".into()).cell(),
                format!("{:.2}", cumulative_hours)
                    .cell()
                    .justify(Justify::Right),
            ]
            .row(),
        );
    }

    Ok(rows.table())
}
