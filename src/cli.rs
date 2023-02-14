use crate::redmine::{
    Activities, Activity, CustomField, CustomValue, NewTimeEntry, Project, Projects,
};
use crate::track::Config;
use crate::{redmine, track};
use anyhow::anyhow;
use chrono::{Datelike, Duration};
use clap::Parser;
use cli_table::print_stdout;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Password};
use regex::Regex;
use std::str::FromStr;
use url::Url;

#[derive(Parser, Debug, Clone)]
#[clap(name = "track", about = "Track your time with redmine.")]
pub struct Cli {
    #[arg(long = "yesterday", short = 'y', help = "Create entry for yesterday.")]
    yesterday: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Parser, Debug, Clone)]
enum Command {
    #[command(name = "login", about = "Login to your account.")]
    Login {
        #[arg(long = "user", short = 'u', help = "The name of your Redmine user.")]
        user: String,
        #[arg(
            long = "baseUrl",
            short = 'b',
            help = "The baseUrl of your redmine installation."
        )]
        base_url: String,
    },
    #[command(
        name = "list",
        about = "List your time entries for today, yesterday or this week."
    )]
    List {
        #[arg(
            long = "yesterday",
            short = 'y',
            help = "Show only time entries for yesterday."
        )]
        yesterday: bool,

        #[arg(
            long = "week",
            short = 'w',
            help = "Show a summary of the weekly activity."
        )]
        week: bool,

        #[arg(
            long = "project",
            short = 'p',
            help = "Show a summary of the weekly activity per project."
        )]
        project: bool,
    },
}

pub fn run(cli: Cli, config: Option<Config>) -> Result<(), anyhow::Error> {
    match (cli.command, config) {
        (None, Some(config)) => {
            let client = redmine::request::Client::new(config);

            let (project, issue) = match ask_for_issue() {
                None => {
                    let projects = client.get_projects()?;
                    let project = select_project(projects);
                    (project, None)
                }

                issue => (None, issue),
            };

            let comment = ask_for_comment();
            let hours = match analyze_comments(comment.clone()) {
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
                None => ask_for_hours(),
            };
            let activities = client.get_activities()?;
            let activity = select_activity(activities);
            let custom_fields = client.get_custom_fields()?;

            let mut custom_values = vec![];
            for field in custom_fields.custom_fields {
                if field.is_for_time_entry() && field.is_required() {
                    if let Some(value) = ask_for_custom_field(field)? {
                        custom_values.push(value)
                    }
                }
            }

            let today = match cli.yesterday {
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

            let _result = client.create_time_entry(new_entry)?;

            println!("Entry successfully created");

            Ok(())
        }
        (
            Some(Command::List {
                yesterday,
                week,
                project,
            }),
            Some(config),
        ) => {
            let client = redmine::request::Client::new(config);
            let today = chrono::Local::now();
            let day = match yesterday {
                true => chrono::Local::now() - Duration::days(1),
                false => chrono::Local::now(),
            };

            let (from, to) = match week {
                true => {
                    let weekday = today.weekday();
                    let start = today - Duration::days(weekday.num_days_from_monday() as i64);
                    let end = today + Duration::days(weekday.num_days_from_sunday() as i64);
                    (start, Some(end))
                }
                false => (day, None),
            };

            let time_entries = client.get_time_entries(from, to)?;
            match (week, project) {
                (_, true) => {
                    let table = track::view::print_table(&time_entries.time_entries)?;
                    print_stdout(table)?;
                    Ok(())
                }
                (true, _) => {
                    let table = track::view::view_weekday_working_hours(time_entries)?;
                    print_stdout(table)?;
                    Ok(())
                }
                _ => {
                    let table = track::view::view_time_entries(time_entries)?;
                    print_stdout(table)?;
                    Ok(())
                }
            }
        }
        (Some(Command::Login { user, base_url }), _) => {
            let pw = Password::new().with_prompt("Password").interact()?;
            let client = reqwest::blocking::Client::new();
            let url = Url::parse(&*base_url)?;
            let copied_url = url.clone();
            let user = redmine::request::login(client, url, user, pw)?;
            let config = Config::new(copied_url, user)?;
            config.store()?;
            println!("You have successfully logged in! A config file has been created at ~/.track");
            Ok(())
        }
        (_, None) => Err(anyhow!(
            "Hi, you don't seem to have logged in yet. Please use \n\n    `track login` \n\n"
        )),
    }
}

fn select_project(projects: Projects) -> Option<Project> {
    let selections: Vec<String> = projects.projects.iter().map(|p| p.name.clone()).collect();

    let selection = dialoguer::FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Please choose the project")
        .items(&selections[..])
        .default(0)
        .interact_opt()
        .unwrap();

    if let Some(selection) = selection {
        Some(projects.projects[selection].clone())
    } else {
        None
    }
}

fn select_activity(activities: Activities) -> Activity {
    let selections: Vec<String> = activities
        .activities
        .iter()
        .map(|a| a.name.clone())
        .collect();

    let default = activities
        .activities
        .iter()
        .position(|a| a.is_default.unwrap_or(false))
        .unwrap_or(0);

    let selection = dialoguer::Select::new()
        .with_prompt("Activity")
        .items(&selections[..])
        .default(default)
        .interact()
        .unwrap();

    activities.activities[selection].clone()
}

fn ask_for_issue() -> Option<i32> {
    let i: String = Input::new()
        .with_prompt("Issue (leave empty for project only)")
        .allow_empty(true)
        .validate_with(|v: &String| {
            let re = Regex::new(r"\d+").unwrap();
            if v.is_empty() || re.is_match(v) {
                Ok(())
            } else {
                Err("Please insert a valid issue number.")
            }
        })
        .interact()
        .unwrap();

    i32::from_str(&*i).ok()
}

fn ask_for_comment() -> String {
    Input::new().with_prompt("Comment").interact().unwrap()
}

fn ask_for_hours() -> f64 {
    Input::new().with_prompt("Hours").interact().unwrap()
}

fn ask_for_custom_field(field: CustomField) -> anyhow::Result<Option<CustomValue>> {
    match &*field.field_format {
        "bool" => {
            let result = Confirm::new().with_prompt(field.name).interact()?;

            Ok(Some(CustomValue {
                id: field.id,
                value: match result {
                    true => "1".to_string(),
                    false => "0".to_string(),
                },
            }))
        }

        "string" => {
            let result = Input::new().with_prompt(field.name).interact()?;
            Ok(Some(CustomValue {
                id: field.id,
                value: result,
            }))
        }

        format => Err(anyhow!("The format {} is unknown, sorry.", format)),
    }
}

fn analyze_comments(input: String) -> Option<(chrono::NaiveTime, chrono::NaiveTime)> {
    let re = Regex::new(r"\s*\d\d?:\d{2}\s*-\s*\d\d?:\d{2}").ok();
    let re = match re {
        None => return None,
        Some(re) => re,
    };

    let m = re.find(&*input)?;
    let filtered: String = m.as_str().chars().filter(|c| !c.is_whitespace()).collect();
    let result: Vec<&str> = filtered.as_str().split("-").collect();
    let left = chrono::NaiveTime::parse_from_str(result[0], "%H:%M").ok();
    let right = chrono::NaiveTime::parse_from_str(result[1], "%H:%M").ok();
    left.and_then(|l| right.map(|r| (l, r)))
}
