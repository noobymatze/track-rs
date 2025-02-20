use anyhow::anyhow;
use clap::Parser;
use dialoguer::Password;
use url::Url;

use crate::redmine;
use crate::track;
use crate::track::Config;

#[derive(Parser, Debug, Clone)]
#[clap(name = "track", about = "Track your time with redmine.")]
pub struct Cli {
    #[arg(long = "yesterday", short = 'y', help = "Create entry for yesterday.")]
    yesterday: bool,
    #[arg(help = "Create entry for specified id.")]
    id: Option<String>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Parser, Debug, Clone)]
enum Command {
    #[command(name = "search", about = "Search for tickets")]
    Search {
        query: String,
        #[arg(long = "direct_track", short = 't', help = "If only one Issue is found, start track.", default_value = "false")]
        direct_track: bool
    },
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
    List(ListArgs),
}

#[derive(Parser, Debug, Clone)]
struct ListArgs {
    /// Show weekly overview, including all issues.
    #[arg(long = "issues", short = 'i')]
    with_issues: bool,

    /// Show time entries from the previous week or day.
    #[arg(long = "previous", short = 'p')]
    previous: bool,

    /// Show a summary of the weekly activity.
    #[arg(long = "week", short = 'w')]
    week: bool,
}

pub fn run(cli: Cli, config: Option<Config>) -> Result<(), anyhow::Error> {
    match (cli.command, config) {
        (Some(Command::Login { user, base_url }), _) => {
            let pw = Password::new().with_prompt("Password").interact()?;
            let client = reqwest::blocking::Client::new();
            let url = Url::parse(&base_url)?;
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
        (None, Some(config)) => {
            let client = redmine::request::Client::new(config);
            track::track(&client, cli.yesterday, cli.id)
        }
        (Some(Command::List(args)), Some(config)) => {
            let client = redmine::request::Client::new(config);
            track::list(&client, args.with_issues, args.previous, args.week)
        }
        (Some(Command::Search { query , direct_track}), Some(config)) => {
            let client = redmine::request::Client::new(config);
            track::search(&client, query, direct_track)
        }
    }
}
