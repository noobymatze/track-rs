mod redmine;
mod track;

use crate::track::Config;

fn main() -> Result<(), anyhow::Error> {
    let config = Config::load()?;
    if let Some(config) = config {
        let client = redmine::request::Client::new(config);
        let time_entries = client.get_time_entries()?;

        let table = track::view::view_time_entries(time_entries)?;
        table.print_stdout()?;
    }

    Ok(())
}
