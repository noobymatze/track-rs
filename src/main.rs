mod cli;
mod redmine;
mod track;

use clap::Parser;
use crate::cli::Cli;
use crate::track::Config;

fn main() -> Result<(), anyhow::Error> {
    let options = Cli::parse();
    let config = Config::load()?;
    cli::run(options, config)
}
