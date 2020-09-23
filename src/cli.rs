use crate::redmine::{Activities, Activity, Project, Projects};
use crate::track::Config;
use crate::{redmine, track};
use anyhow::anyhow;
use dialoguer::Input;
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
    Login,
    #[structopt(name = "list", about = "List your time entries for the current day.")]
    List,
}

pub fn run(options: Options, config: Option<Config>) -> Result<(), anyhow::Error> {
    match (options.command, config) {
        (None, Some(config)) => {
            let client = redmine::request::Client::new(config);

            if let Some(issue) = ask_for_issue() {
                let comment = ask_for_comment();
                let hours = ask_for_hours();
                let activities = client.get_activities()?;
                let activity = select_activity(activities);


            } else {
                let projects = client.get_projects()?;
                if let Some(project) = select_project(projects) {
                    println!("{}", project.name)
                }
            }

            Ok(())
        }
        (Some(Command::List), Some(config)) => {
            let client = redmine::request::Client::new(config);
            let time_entries = client.get_time_entries()?;

            let table = track::view::view_time_entries(time_entries)?;
            table.print_stdout()?;
            Ok(())
        }
        (Some(Command::Login), _) => Ok(()),
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
    let default = 0;

    let selection = dialoguer::Select::new()
        .with_prompt("Please choose the project")
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
