use std::fmt::Display;

use derive_more::derive::From;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    Custom(String),
    UnableToWriteConfigToFile(std::io::Error),
    FailedToGetConfigDirectory,
    UnableToConvertToToml(toml::ser::Error),
    UnableToConvertFromToml(toml::de::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}
