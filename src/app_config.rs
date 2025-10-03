extern crate config;

use anyhow::Result;
use config::{Config, File};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)] // Add Debug for easier printing
pub struct ParserConfig {
    pub bookmark: String,
    pub highlight: String,
    pub note: String,
}

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub parser: ParserConfig,
}

impl AppConfig {
    pub fn new(config_file: Option<PathBuf>) -> Result<Self, config::ConfigError> {
        let mut builder = Config::builder()
            .set_default("parser.bookmark", "- Votre signet")?
            .set_default("parser.highlight", "- Votre surlignement")?
            .set_default("parser.note", "- Votre note")?;

        if let Some(config_file_path) = config_file {
            builder = builder.add_source(File::from(config_file_path));
        }

        // Build and deserialize the entire configuration into our struct
        builder.build()?.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_default_values() {
        // 1. Initialize configuration without providing a specific file.
        let config = AppConfig::new(None).unwrap();

        // 2. Check values directly on the returned struct instance.
        assert_eq!(config.parser.bookmark, "- Votre signet");
        assert_eq!(config.parser.highlight, "- Votre surlignement");
        assert_eq!(config.parser.note, "- Votre note");
    }

    #[test]
    fn test_custom_configuration() {
        // 1. Initialize configuration using the specific English config file.
        let config_path = Some(PathBuf::from("src/resources/english_config.toml"));
        let config = AppConfig::new(config_path).unwrap();

        // 2. Assert that the values from the file have correctly overridden the defaults.
        assert_eq!(config.parser.bookmark, "- Your Bookmark");
        assert_eq!(config.parser.highlight, "- Your Highlight");
        assert_eq!(config.parser.note, "- Your Note");
    }
}
