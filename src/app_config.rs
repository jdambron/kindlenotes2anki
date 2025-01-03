extern crate config;

use anyhow::{Context, Result};
use config::{Config, File};
use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

fn app_config() -> &'static RwLock<Config> {
    static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
    CONFIG.get_or_init(|| RwLock::new(Config::default()))
}

#[derive(Deserialize)]
pub struct AppConfig {}

impl AppConfig {
    pub fn init(config_file: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        let mut builder = Config::builder()
            .set_default("parser.bookmark", "- Votre signet")?
            .set_default("parser.highlight", "- Votre surlignement")?
            .set_default("parser.note", "- Votre note")?;
        if let Some(config_file_path) = config_file {
            builder = builder.add_source(File::from(config_file_path));
        }
        let settings = builder.build().context("Failed to load config file")?;
        // Save Config to RwLoc
        {
            let mut w = app_config().write()?;
            *w = settings;
        }
        Ok(())
    }

    // Get a single value
    pub fn get<'de, T>(key: &'de str) -> Result<T, Box<dyn Error>>
    where
        T: serde::Deserialize<'de>,
    {
        Ok(app_config().read()?.get::<T>(key)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::path::PathBuf;

    #[test]
    #[serial]
    fn test_default_values() {
        // Initialize configuration
        AppConfig::init(None).unwrap();

        // Check value with get
        assert_eq!(
            AppConfig::get::<String>("parser.bookmark").unwrap(),
            "- Votre signet"
        );
        assert_eq!(
            AppConfig::get::<String>("parser.highlight").unwrap(),
            "- Votre surlignement"
        );
        assert_eq!(
            AppConfig::get::<String>("parser.note").unwrap(),
            "- Votre note"
        );
    }

    #[test]
    #[serial]
    fn test_custom_configuration() {
        // Initialize configuration
        AppConfig::init(Some(PathBuf::from("src/resources/english_config.toml"))).unwrap();

        // Check value with get
        assert_eq!(
            AppConfig::get::<String>("parser.bookmark").unwrap(),
            "- Your Bookmark"
        );
        assert_eq!(
            AppConfig::get::<String>("parser.highlight").unwrap(),
            "- Your Highlight"
        );
        assert_eq!(
            AppConfig::get::<String>("parser.note").unwrap(),
            "- Your Note"
        );
    }
}
