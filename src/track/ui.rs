use anyhow::anyhow;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};
use regex::Regex;
use std::str::FromStr;

use crate::redmine::{Activities, Activity, CustomField, CustomValue, Project, Projects};

pub fn select_project(projects: Projects) -> Option<Project> {
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

pub fn select_activity(activities: Activities) -> Activity {
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

pub fn ask_for_issue() -> Option<i32> {
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

    i32::from_str(&i).ok()
}

pub fn ask_for_comment() -> String {
    Input::new().with_prompt("Comment").interact().unwrap()
}

pub fn ask_for_hours() -> f64 {
    Input::new().with_prompt("Hours").interact().unwrap()
}

pub fn ask_for_custom_field(field: CustomField) -> anyhow::Result<Option<CustomValue>> {
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

pub fn analyze_comments(input: String) -> Option<(chrono::NaiveTime, chrono::NaiveTime)> {
    let re = Regex::new(r"\s*\d\d?:\d{2}\s*-\s*\d\d?:\d{2}").ok();
    let re = match re {
        None => return None,
        Some(re) => re,
    };

    let m = re.find(&input)?;
    let filtered: String = m.as_str().chars().filter(|c| !c.is_whitespace()).collect();
    let result: Vec<&str> = filtered.as_str().split('-').collect();
    let left = chrono::NaiveTime::parse_from_str(result[0], "%H:%M").ok();
    let right = chrono::NaiveTime::parse_from_str(result[1], "%H:%M").ok();
    left.and_then(|l| right.map(|r| (l, r)))
}
