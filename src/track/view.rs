use std::collections::HashMap;

use chrono::{Datelike, Weekday};
use cli_table::{Cell, CellStruct, Row, RowStruct, Style, Table, TableStruct};
use cli_table::format::Justify;

use crate::redmine::{TimeEntries, TimeEntry};

pub fn view_time_entries(time_entries: TimeEntries) -> Result<TableStruct, anyhow::Error> {
    let mut entries = time_entries.time_entries;
    entries.sort_by_key(|t| t.id);

    let total_hours: f64 = entries.iter().map(|t| t.hours).sum();
    let header_row = vec![
        "Project".cell().bold(true),
        "Issue".cell().bold(true),
        format!("Hours (∑ {})", total_hours).cell().bold(true),
        "Comment".cell().bold(true),
    ].row();

    let mut rows = vec![header_row];
    for entry in entries.iter() {
        rows.push(view_time_entry(entry));
    }

    Ok(rows.table())
}

fn view_time_entry(entry: &TimeEntry) -> RowStruct {
    let project = entry.project.clone().name;
    let issue = entry
        .issue
        .clone()
        .map(|i| format!("#{}", i.id))
        .unwrap_or("".to_string());

    let comment = entry.comments.clone();

    vec![
        project.unwrap_or("".into()).cell(),
        issue.cell(),
        entry.hours.to_string().cell().justify(Justify::Right),
        comment.unwrap_or("".into()).cell()
    ].row()
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