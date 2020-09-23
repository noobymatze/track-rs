use anyhow::{anyhow};
use crate::track::Config;
use structopt::StructOpt;
use crate::{redmine, track};

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "track", about = "Track your time with redmine.")]
pub struct Options {
    #[structopt(subcommand)]
    command: Option<Command>
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
        (None, Some(_)) => {
            Ok(())
        }
        (Some(Command::List), Some(config)) => {
            let client = redmine::request::Client::new(config);
            let time_entries = client.get_time_entries()?;

            let table = track::view::view_time_entries(time_entries)?;
            table.print_stdout()?;
            Ok(())
        },
        (Some(Command::Login), _) => {
            Ok(())
        },
        (_, None) => {
            Err(anyhow!("You don't seem to have logged in yet, please use `track login`."))
        },
    }
}
