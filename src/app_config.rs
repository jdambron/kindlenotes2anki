use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

fn default_ignored() -> Vec<String> {
    vec![
        // Junk notices Kindle injects when a book's clipping limit is hit
        "<Vous avez atteint la limite maximale".to_owned(),
        "<You have reached the clipping limit".to_owned(),
    ]
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ParserConfig {
    pub bookmark: String,
    pub highlight: String,
    pub note: String,
    #[serde(default = "default_ignored")]
    pub ignored: Vec<String>,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            bookmark: "- Votre signet".to_owned(),
            highlight: "- Votre surlignement".to_owned(),
            note: "- Votre note".to_owned(),
            ignored: default_ignored(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AnkiConfig {
    pub deck: String,
    pub model: String,
    pub front_field: String,
    pub back_field: String,
    pub url: String,
}

impl Default for AnkiConfig {
    fn default() -> Self {
        Self {
            deck: "Kindle".to_owned(),
            model: "Basique".to_owned(),
            front_field: "Recto".to_owned(),
            back_field: "Verso".to_owned(),
            url: "http://localhost:8765".to_owned(),
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct AppConfig {
    #[serde(default)]
    pub parser: ParserConfig,
    #[serde(default)]
    pub anki: AnkiConfig,
}

impl AppConfig {
    pub fn new(config_file: Option<PathBuf>) -> Result<Self> {
        match config_file {
            None => Ok(Self::default()),
            Some(path) => {
                let contents = std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read config file: {}", path.display()))?;
                let contents = contents.strip_prefix('\u{feff}').unwrap_or(&contents);
                toml::from_str(contents)
                    .with_context(|| format!("Failed to parse config file: {}", path.display()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_values() {
        let config = AppConfig::new(None).unwrap();

        assert_eq!(config.parser.bookmark, "- Votre signet");
        assert_eq!(config.parser.highlight, "- Votre surlignement");
        assert_eq!(config.parser.note, "- Votre note");
        assert_eq!(config.anki.deck, "Kindle");
        assert_eq!(config.anki.model, "Basique");
        assert_eq!(config.anki.front_field, "Recto");
        assert_eq!(config.anki.back_field, "Verso");
        assert_eq!(config.anki.url, "http://localhost:8765");
        assert_eq!(
            config.parser.ignored,
            [
                "<Vous avez atteint la limite maximale",
                "<You have reached the clipping limit"
            ]
        );
    }

    #[test]
    fn test_custom_configuration() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", include_str!("resources/english_config.toml")).unwrap();
        let config = AppConfig::new(Some(file.path().to_path_buf())).unwrap();

        assert_eq!(config.parser.bookmark, "- Your Bookmark");
        assert_eq!(config.parser.highlight, "- Your Highlight");
        assert_eq!(config.parser.note, "- Your Note");
        assert_eq!(config.anki.deck, "Kindle");
        assert_eq!(config.anki.model, "Basic");
        assert_eq!(config.anki.front_field, "Front");
        assert_eq!(config.anki.back_field, "Back");
        assert_eq!(config.anki.url, "http://localhost:8765");
    }

    #[test]
    fn test_partial_parser_keeps_anki_defaults() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[parser]
bookmark = "- Your Bookmark"
highlight = "- Your Highlight"
note = "- Your Note"
"#
        )
        .unwrap();

        let config = AppConfig::new(Some(file.path().to_path_buf())).unwrap();
        assert_eq!(config.parser.highlight, "- Your Highlight");
        assert_eq!(config.anki, AnkiConfig::default());
    }

    #[test]
    fn test_partial_anki_keeps_parser_defaults() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[anki]
deck = "Clippings"
model = "Basic"
front_field = "Front"
back_field = "Back"
url = "http://127.0.0.1:9999"
"#
        )
        .unwrap();

        let config = AppConfig::new(Some(file.path().to_path_buf())).unwrap();
        assert_eq!(config.parser, ParserConfig::default());
        assert_eq!(config.anki.deck, "Clippings");
        assert_eq!(config.anki.url, "http://127.0.0.1:9999");
    }

    #[test]
    fn test_custom_ignored_patterns_override_defaults() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[parser]
bookmark = "- Your Bookmark"
highlight = "- Your Highlight"
note = "- Your Note"
ignored = ["<skip me>"]
"#
        )
        .unwrap();

        let config = AppConfig::new(Some(file.path().to_path_buf())).unwrap();
        assert_eq!(config.parser.ignored, ["<skip me>"]);
    }

    #[test]
    fn test_missing_config_file() {
        let err = AppConfig::new(Some(PathBuf::from("does-not-exist.toml"))).unwrap_err();
        assert!(err.to_string().contains("Failed to read config file"));
    }

    #[test]
    fn test_invalid_toml() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "not = [valid").unwrap();
        let err = AppConfig::new(Some(file.path().to_path_buf())).unwrap_err();
        assert!(err.to_string().contains("Failed to parse config file"));
    }
}
