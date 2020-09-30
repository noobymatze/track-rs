use crate::redmine::{TimeEntries, TimeEntry};
use cli_table::format::{CellFormat, Justify};
use cli_table::{Cell, Row, Table};
use std::collections::HashMap;
use std::io;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::{Terminal, symbols};
use termion::raw::IntoRawMode;
use tui::layout::{Layout, Direction, Constraint};
use tui::widgets::{BarChart, Block, Borders, Dataset, Chart, Axis, GraphType};
use tui::style::{Style, Modifier, Color};
use termion::event::{Key, Event};
use termion::input::{Events, TermRead};
use chrono::{Datelike, Weekday};
use tui::text::Span;

pub fn view_time_entries_week(time_entries: TimeEntries) -> anyhow::Result<()> {
    // group by spent_on
    let mut map = HashMap::new();
    for t in time_entries.time_entries {
        let date = chrono::NaiveDate::parse_from_str(&*t.spent_on, "%Y-%m-%d")?;
        let x = date.weekday();
        let mut v = map.entry(x).or_insert(0.0);
        *v += t.hours;
    }


    let stdin = io::stdin();
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app: Vec<(f64, f64)> = vec![
        (0.0, *map.get(&Weekday::Mon).unwrap_or(&0.0)),
        (1.0, *map.get(&Weekday::Tue).unwrap_or(&0.0)),
        (2.0, *map.get(&Weekday::Wed).unwrap_or(&0.0)),
        (3.0, *map.get(&Weekday::Thu).unwrap_or(&0.0)),
        (4.0, *map.get(&Weekday::Fri).unwrap_or(&0.0)),
        (5.0, *map.get(&Weekday::Sat).unwrap_or(&0.0)),
        (6.0, *map.get(&Weekday::Sun).unwrap_or(&0.0)),
    ];

    terminal.draw(|f| {

        let dataset = Dataset::default()
            .name("Total")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Cyan))
            .graph_type(GraphType::Line)
            .data(&app);
        let data = &vec![(0.0, 8.0), (1.0, 8.0), (2.0, 8.0), (3.0, 8.0), (4.0, 8.0), (5.0, 8.0), (6.0, 8.0)];
        let base_line_dataset = Dataset::default()
            .name("Bound")
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(Color::Magenta))
            .graph_type(GraphType::Line)
            .data(&data);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.size());

        let chart = Chart::new(vec![base_line_dataset, dataset])
            .block(
                Block::default()
                    .title(Span::styled(
                        "Week stats",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("Day")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, 6.0])
                    .labels(vec![
                        Span::styled("Monday", Style::default()),
                        Span::styled("Tuesday", Style::default()),
                        Span::styled("Wednesday", Style::default()),
                        Span::styled("Thursday", Style::default()),
                        Span::styled("Friday", Style::default()),
                        Span::styled("Saturday", Style::default()),
                        Span::styled("Sunday", Style::default()),
                    ]),
            )
            .y_axis(
                Axis::default()
                    .title("Hours")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, 12.0])
                    .labels(vec![
                        Span::styled("0h", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("1h", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("2h", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("3h", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("4h", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("5h", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("6h"),
                        Span::styled("7h", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("8h", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("9h", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("10h", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("11h", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled("12h", Style::default().add_modifier(Modifier::BOLD)),
                    ]),
            );
        f.render_widget(chart, chunks[0]);

        //let barchart = BarChart::default()
        //    .block(Block::default().title("Statistics Week").borders(Borders::ALL))
        //    .data(&app)
        //    .bar_gap(3)
        //    .max(8)
        //    .bar_width(11)
        //    .bar_style(Style::default().fg(Color::Yellow))
        //    .value_style(Style::default().fg(Color::Black).bg(Color::Yellow))
        //    .label_style(Style::default());
        // f.render_widget(barchart, chunks[0]);

        //let chunks = Layout::default()
        //    .direction(Direction::Horizontal)
        //    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        //    .split(chunks[1]);

        //let barchart = BarChart::default()
        //    .block(Block::default().title("Data2").borders(Borders::ALL))
        //    .data(&v.as_slice())
        //    .bar_width(5)
        //    .bar_gap(3)
        //    .bar_style(Style::default().fg(Color::Green))
        //    .value_style(
        //        Style::default()
        //            .bg(Color::Green)
        //            .add_modifier(Modifier::BOLD),
        //    );
        //f.render_widget(barchart, chunks[0]);

        //let barchart = BarChart::default()
        //    .block(Block::default().title("Data3").borders(Borders::ALL))
        //    .data(&v)
        //    .bar_style(Style::default().fg(Color::Red))
        //    .bar_width(7)
        //    .bar_gap(0)
        //    .value_style(Style::default().bg(Color::Red))
        //    .label_style(
        //        Style::default()
        //            .fg(Color::Cyan)
        //            .add_modifier(Modifier::ITALIC),
        //    );
        //f.render_widget(barchart, chunks[1]);
    })?;

    for c in stdin.events() {
        let c = c?;
        match c {
            Event::Key(Key::Char('q')) => break,
            _ => {}
        }
    }


    Ok(())
}

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
