extern crate config;

use config::Config;
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
    pub fn init(default_config: Option<&str>) -> Result<(), Box<dyn Error>> {
        let mut settings = Config::new();
        if let Some(config_contents) = default_config {
            settings.merge(config::File::from_str(
                &config_contents,
                config::FileFormat::Toml,
            ))?;
        }
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

    // Merge config from another file
    pub fn merge_config(config_file: Option<PathBuf>) -> Result<(), Box<dyn Error>> {
        // Merge settings with config file if there is one
        if let Some(config_file_path) = config_file {
            {
                CONFIG
                    .write()?
                    .merge(config::File::from(config_file_path))?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::path::Path;

    #[test]
    #[serial]
    fn verify_get() {
        // Initialize configuration
        let config_contents = include_str!("resources/test_config.toml");
        AppConfig::init(Some(config_contents)).unwrap();

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
        let config_contents = include_str!("resources/test_config.toml");
        AppConfig::init(Some(config_contents)).unwrap();
        let path = Path::new("src/resources/english_config.toml");
        let mut path_buf = PathBuf::new();
        path_buf.push(path);
        AppConfig::merge_config(Some(path_buf)).unwrap();

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
