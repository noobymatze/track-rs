use crate::redmine::{
    Activities, Activity, CustomField, CustomValue, NewTimeEntry, Project, Projects,
};
use crate::track::Config;
use crate::{redmine, track};
use anyhow::anyhow;
use dialoguer::{Confirm, Input};
use regex::Regex;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "track", about = "Track your time with redmine.")]
pub struct Options {
    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(StructOpt, Debug, Clone)]
enum Command {
    #[structopt(name = "login", about = "Login to your account.")]
    Login {
        #[structopt(name = "user", short = "u")]
        user: String,
    },
    #[structopt(name = "list", about = "List your time entries for the current day.")]
    List,
}

pub fn run(options: Options, config: Option<Config>) -> Result<(), anyhow::Error> {
    match (options.command, config) {
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
            let hours = ask_for_hours();
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

            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let new_entry = NewTimeEntry {
                issue_id: issue,
                project_id: project.map(|p| p.id),
                hours,
                comments: comment,
                activity_id: activity.id,
                custom_fields: custom_values,
                spent_on: today,
            };

            println!("{:?}", new_entry);

            let result = client.create_time_entry(new_entry)?;
            Ok(())
        }
        (Some(Command::List), Some(config)) => {
            let client = redmine::request::Client::new(config);
            let time_entries = client.get_time_entries()?;

            let table = track::view::view_time_entries(time_entries)?;
            table.print_stdout()?;
            Ok(())
        }
        (Some(Command::Login { user }), _) => Ok(()),
        (_, None) => Err(anyhow!(
            "You don't seem to have logged in yet, please use `track login`."
        )),
    }
}

fn select_project(projects: Projects) -> Option<Project> {
    let selections: Vec<String> = projects.projects.iter().map(|p| p.name.clone()).collect();

    let selection = dialoguer::Select::new()
        .with_prompt("Please choose the project")
        .items(&selections[..])
        .default(0)
        .paged(true)
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
        .paged(true)
        .interact()
        .unwrap();

    activities.activities[selection].clone()
}

fn ask_for_issue() -> Option<i32> {
    let i: String = Input::new()
        .with_prompt("Issue you have been working on")
        .validate_with(|v: &str| {
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
