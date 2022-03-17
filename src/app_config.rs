extern crate config;

use config::{Config, File};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::error::Error;
use std::path::PathBuf;
use std::sync::RwLock;

lazy_static! {
    static ref CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

#[derive(Deserialize)]
pub struct Parser {
    pub bookmark: String,
    pub highlight: String,
    pub note: String,
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub parser: Parser,
}

impl AppConfig {
    pub fn init(config_file: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        let settings = match config_file {
            None => Config::builder()
                .set_default("parser.bookmark", "- Votre signet")?
                .set_default("parser.highlight", "- Votre surlignement")?
                .set_default("parser.note", "- Votre note")?
                .build()
                .unwrap(),
            Some(config_file_path) => Config::builder()
                .set_default("parser.bookmark", "- Votre signet")?
                .set_default("parser.highlight", "- Votre surlignement")?
                .set_default("parser.note", "- Votre note")?
                .add_source(File::from(config_file_path))
                .build()
                .unwrap(),
        };
        // Save Config to RwLoc
        {
            let mut w = CONFIG.write()?;
            *w = settings;
        }
        Ok(())
    }

    // Get a single value
    pub fn get<'de, T>(key: &'de str) -> Result<T, Box<dyn Error>>
    where
        T: serde::Deserialize<'de>,
    {
        Ok(CONFIG.read()?.get::<T>(key)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::path::PathBuf;

    #[test]
    #[serial]
    fn verify_get() {
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
    fn verify_merge() {
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
