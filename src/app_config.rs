use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct ParserConfig {
    pub bookmark: String,
    pub highlight: String,
    pub note: String,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            bookmark: "- Votre signet".to_owned(),
            highlight: "- Votre surlignement".to_owned(),
            note: "- Votre note".to_owned(),
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct AppConfig {
    pub parser: ParserConfig,
}

impl AppConfig {
    pub fn new(config_file: Option<PathBuf>) -> Result<Self> {
        match config_file {
            None => Ok(Self::default()),
            Some(path) => {
                let contents = std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read config file: {}", path.display()))?;
                toml::from_str(&contents)
                    .with_context(|| format!("Failed to parse config file: {}", path.display()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_default_values() {
        let config = AppConfig::new(None).unwrap();

        assert_eq!(config.parser.bookmark, "- Votre signet");
        assert_eq!(config.parser.highlight, "- Votre surlignement");
        assert_eq!(config.parser.note, "- Votre note");
    }

    #[test]
    fn test_custom_configuration() {
        let config_path = Some(PathBuf::from("src/resources/english_config.toml"));
        let config = AppConfig::new(config_path).unwrap();

        assert_eq!(config.parser.bookmark, "- Your Bookmark");
        assert_eq!(config.parser.highlight, "- Your Highlight");
        assert_eq!(config.parser.note, "- Your Note");
    }
}
