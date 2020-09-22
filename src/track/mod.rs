pub mod view;

use serde::{Deserialize, Serialize};
use std::fs::File;

use std::io::BufReader;

use crate::track::Error::HomeDirNotFound;
use thiserror::Error;
use url::Url;

#[derive(Serialize, Deserialize, Debug, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub key: String,
    pub base_url: Url,
    pub login: String,
    pub user_id: i32,
}

#[derive(Error, Debug)]
enum Error {
    #[error("Your home directory could not be found, sorry.")]
    HomeDirNotFound(),
}

impl Config {
    pub fn load() -> Result<Option<Self>, anyhow::Error> {
        let home_dir = dirs::home_dir().ok_or(HomeDirNotFound())?;
        let track_file = home_dir.join(".track");
        let file = File::open(track_file)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(Some(config))
    }
}
