pub mod report;

use serde::{Deserialize, Serialize};
use std::fs::File;

use std::io::{BufReader, BufWriter};

use crate::redmine::{CustomField, User};
use crate::track::Error::{ApiKeyMissing, HomeDirNotFound};
use std::io;
use thiserror::Error;
use url::Url;

/// A `Config` defines all parameters necessary, to connect to a Redmine server.
///
/// It is stored in the users home directory as a .track file.
#[derive(Serialize, Deserialize, Debug, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub key: String,
    pub base_url: Url,
    pub login: String,
    pub user_id: i32,
    #[serde(default)]
    pub custom_fields: Vec<CustomField>,
}

/// This type represents any error, that can happen while loading or storing
/// a `Config`.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Your home directory could not be found.")]
    HomeDirNotFound,
    #[error("The configuration could not be read or written.")]
    Io(#[from] io::Error),
    #[error("The configuration could not be (de)/serialized.")]
    Json(#[from] serde_json::Error),
    #[error("The API key is missing, please create one in your user settings.")]
    ApiKeyMissing,
}

impl Config {
    /// Returns a new `Config` based on the `base_url` and `user`.
    ///
    /// ## Example
    ///
    /// ```
    /// let config = Config::new("https://myredmine.com", user);
    /// assert(config.user_id, 1);
    /// ```
    pub fn new(base_url: Url, user: User) -> Result<Self, Error> {
        let config = Config {
            key: user.api_key.ok_or(ApiKeyMissing)?,
            base_url,
            login: user.login,
            user_id: user.id,
            custom_fields: vec![],
        };

        Ok(config)
    }

    /// Load the configuration from `~/.track`.
    ///
    /// ## Example
    ///
    /// ```
    /// let result = Config::load();
    /// ```
    ///
    pub fn load() -> Result<Option<Self>, Error> {
        let home_dir = dirs::home_dir().ok_or(Error::HomeDirNotFound)?;
        let track_file = home_dir.join(".track");

        if !track_file.exists() {
            return Ok(None);
        }

        let file = File::open(track_file)?;
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader)?;
        Ok(Some(config))
    }

    /// Store this configuration in `~/.track`.
    ///
    /// ## Example
    ///
    /// ```
    /// let config = Config::new("https://myredmine.com", user);
    /// config.store();
    /// ```
    ///
    pub fn store(&self) -> Result<(), Error> {
        let home_dir = dirs::home_dir().ok_or(HomeDirNotFound)?;
        let track_file = home_dir.join(".track");
        let file = File::create(track_file)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;

        Ok(())
    }
}
