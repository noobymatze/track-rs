mod redmine;
mod track;
mod cli;

use structopt::StructOpt;
use crate::track::Config;
use crate::cli::Options;


fn main() -> Result<(), anyhow::Error> {
    let options = Options::from_args();
    let config = Config::load()?;
    cli::run(options, config)
}
