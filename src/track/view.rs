use crate::redmine::{TimeEntries, TimeEntry};
use cli_table::format::{CellFormat, Justify};
use cli_table::{Cell, Row, Table};

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

    let justify_right = CellFormat::builder()
        .justify(Justify::Right)
        .build();

    let cells = vec![
        Cell::new(&project.unwrap_or("".to_string()), Default::default()),
        Cell::new(&issue, Default::default()),
        Cell::new(&hours, justify_right),
        Cell::new(&comment, Default::default()),
    ];

    Row::new(cells)
}
