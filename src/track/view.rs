use crate::redmine::{TimeEntries, TimeEntry};
use chrono::{Datelike, Weekday};
use cli_table::format::{CellFormat, Justify};
use cli_table::{Cell, Row, Table};
use std::collections::HashMap;

pub fn view_time_entries(time_entries: TimeEntries) -> Result<Table, anyhow::Error> {
    let bold = CellFormat::builder().bold(true).build();
    let mut entries = time_entries.time_entries;
    entries.sort_by_key(|t| t.id);
    let mut rows: Vec<Row> = entries.iter().map(|entry| view_time_entry(entry)).collect();
    let total_hours: f64 = entries.iter().map(|t| t.hours).sum();

    let header_row = Row::new(vec![
        Cell::new("Project", bold),
        Cell::new("Issue", bold),
        Cell::new(&format!("Hours (∑ {})", total_hours), bold),
        Cell::new("Comment", bold),
    ]);

    rows.reverse();
    rows.push(header_row);
    rows.reverse();

    let result = Table::new(rows, Default::default())?;

    Ok(result)
}

fn view_time_entry(entry: &TimeEntry) -> Row {
    let project = entry.project.clone().name;
    let issue = entry
        .issue
        .clone()
        .map(|i| format!("#{}", i.id))
        .unwrap_or("".to_string());
    let hours = entry.hours.to_string();
    let comment = entry.comments.clone().unwrap_or("".to_string());

    let justify_right = CellFormat::builder().justify(Justify::Right).build();

    let cells = vec![
        Cell::new(&project.unwrap_or("".to_string()), Default::default()),
        Cell::new(&issue, Default::default()),
        Cell::new(&hours, justify_right),
        Cell::new(&comment, Default::default()),
    ];

    Row::new(cells)
}

pub fn view_weekday_working_hours(time_entries: TimeEntries) -> Result<Table, anyhow::Error> {
    let header_format = CellFormat::builder()
        .bold(true)
        .justify(Justify::Right)
        .build();
    let centered = CellFormat::builder().justify(Justify::Right).build();
    // group by spent_on
    let mut map = HashMap::new();
    for t in time_entries.time_entries {
        let date = chrono::NaiveDate::parse_from_str(&*t.spent_on, "%Y-%m-%d")?;
        let x = date.weekday();
        let v = map.entry(x).or_insert(0.0);
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

    let mut all_headers: Vec<Cell> = headers
        .iter()
        .map(|w| Cell::new(&w.to_string(), header_format))
        .collect();
    all_headers.push(Cell::new("∑", header_format));

    let mut cells: Vec<Cell> = headers
        .iter()
        .map(|w| map.get(w).unwrap_or(&0.0))
        .map(|v| Cell::new(&v.to_string(), centered))
        .collect();

    let sum: f64 = map.values().sum();

    cells.push(Cell::new(&sum.to_string(), centered));

    let rows = vec![Row::new(all_headers), Row::new(cells)];

    let result = Table::new(rows, Default::default())?;

    Ok(result)
}
