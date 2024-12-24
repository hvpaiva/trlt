use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub model: String,
}

impl Config {
    pub fn new(api_key: Option<String>, model: String) -> Result<Self> {
        Ok(Self {
            api_key: Self::api_key(api_key)?,
            model,
        })
    }

    pub fn read_from_file() -> Option<Self> {
        let contents = fs::read_to_string(Config::config_path()).ok()?;
        let config: Config = toml::from_str(&contents).expect("Failed to parse config");

        Some(config)
    }

    pub fn write_to_file(&self) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;

        fs::write(Config::config_path(), contents)?;
        Ok(())
    }

    fn api_key(api_key: Option<String>) -> Result<String> {
        if let Some(api_key) = api_key {
            Ok(api_key)
        } else {
            let mut api_key_value = String::new();
            println!("Provide the OpenAI API key: ");
            io::stdin().read_line(&mut api_key_value)?;

            Ok(api_key_value.trim().to_string())
        }
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .expect("Failed to get config directory")
            .join("trlt.toml")
    }
}
