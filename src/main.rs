mod cli;
mod redmine;
mod track;

use crate::cli::Cli;
use crate::track::Config;
use clap::Parser;

fn main() -> Result<(), anyhow::Error> {
    let options = Cli::parse();
    let config = Config::load()?;
    cli::run(options, config)
}
