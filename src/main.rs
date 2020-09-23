mod cli;
mod redmine;
mod track;

use crate::cli::Options;
use crate::track::Config;
use structopt::StructOpt;

fn main() -> Result<(), anyhow::Error> {
    let options = Options::from_args();
    let config = Config::load()?;
    cli::run(options, config)
}
