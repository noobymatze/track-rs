pub mod view;

use serde::{Deserialize, Serialize};
use std::fs::File;

use std::io::{BufReader, BufWriter};

use crate::redmine::User;
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
    pub fn new(base_url: Url, user: User) -> Self {
        Config {
            key: user.api_key,
            base_url: base_url,
            login: user.login,
            user_id: user.id,
        }
    }
    pub fn load() -> Result<Option<Self>, anyhow::Error> {
        let home_dir = dirs::home_dir().ok_or(HomeDirNotFound())?;
        let track_file = home_dir.join(".track");

        if track_file.exists() {
            let file = File::open(track_file)?;
            let reader = BufReader::new(file);
            let config = serde_json::from_reader(reader)?;
            Ok(Some(config))
        }
        else {
            Ok(None)
        }
    }

    pub fn store(&self) -> anyhow::Result<()> {
        let home_dir = dirs::home_dir().ok_or(HomeDirNotFound())?;
        let track_file = home_dir.join(".track");
        let file = File::create(track_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;

        Ok(())
    }
}
