use crate::redmine::{TimeEntries, TimeEntry};
use cli_table::format::{CellFormat, Justify};
use cli_table::{Cell, Row, Table};
use std::collections::HashMap;
use std::io;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;
use termion::raw::IntoRawMode;
use tui::layout::{Layout, Direction, Constraint};
use tui::widgets::{BarChart, Block, Borders};
use tui::style::{Style, Modifier, Color};
use termion::event::{Key, Event};
use termion::input::{Events, TermRead};

struct App<'a> {
    data: Vec<(&'a str, u64)>
}

pub fn view_time_entries_week(time_entries: TimeEntries) -> anyhow::Result<()> {
    // group by spent_on
    let mut map = HashMap::new();
    for t in time_entries.time_entries {
        let key = t.spent_on.chars().skip(5).collect::<String>();
        let v = map.entry(key).or_insert(0.0);
        *v += t.hours;
    }



    let stdin = io::stdin();
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App {
        data: map.iter().map(|(a, b)| (a.as_str(), b.round() as u64)).collect()
    };

    terminal.draw(|f| {

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.size());

        let barchart = BarChart::default()
            .block(Block::default().title("Data1").borders(Borders::ALL))
            .data(&app.data)
            .bar_width(9)
            .bar_style(Style::default().fg(Color::Yellow))
            .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));
        f.render_widget(barchart, chunks[0]);

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
