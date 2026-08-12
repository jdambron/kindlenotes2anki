use crate::app_config::{AppConfig, ParserConfig};
use crate::note::Note;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const SEPARATOR: &str = "==========";

pub fn parse_clippings(filename: &PathBuf, config: &AppConfig) -> Result<Vec<Note>> {
    let file = File::open(filename)
        .with_context(|| format!("Failed to open clippings file: {}", filename.display()))?;
    let reader = BufReader::new(file);
    let mut notes = Vec::with_capacity(100);
    let mut current_note = Vec::with_capacity(10);
    let prefixes = &config.parser;

    for line in reader.lines() {
        let line = line?;
        if line.starts_with(SEPARATOR) {
            if !current_note.is_empty() {
                if let Some(note) = parse_note(&current_note, prefixes) {
                    notes.push(note);
                }
                current_note.clear();
            }
        } else {
            current_note.push(line);
        }
    }

    if !current_note.is_empty()
        && let Some(note) = parse_note(&current_note, prefixes)
    {
        notes.push(note);
    }

    Ok(notes)
}

fn parse_note(lines: &[String], prefixes: &ParserConfig) -> Option<Note> {
    if lines.is_empty() {
        return None;
    }

    let title = lines[0].trim();
    if is_empty_or_useless_line(title, prefixes) {
        return None;
    }

    let tidied_note: String = lines[1..]
        .iter()
        .filter(|l| !is_empty_or_useless_line(l, prefixes))
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    if tidied_note.is_empty() {
        None
    } else {
        Some(Note {
            title: title.to_owned(),
            tidied_note,
        })
    }
}

fn is_empty_or_useless_line(line: &str, prefixes: &ParserConfig) -> bool {
    line.is_empty()
        || line.starts_with(&prefixes.highlight)
        || line.starts_with(&prefixes.bookmark)
        || line.starts_with(&prefixes.note)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::AppConfig;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn setup_test_config() -> AppConfig {
        AppConfig::new(Some(PathBuf::from("src/resources/test_config.toml"))).unwrap()
    }

    #[test]
    fn test_parse_clippings_empty() {
        let config = setup_test_config();
        let temp_dir = tempdir().expect("Failed to create temporary directory.");
        let file_path = temp_dir.path().join("test_clippings.txt");
        std::fs::File::create(&file_path).expect("Failed to create temporary file.");

        let notes = parse_clippings(&file_path, &config).expect("Failed to parse temporary file.");
        assert!(notes.is_empty());
    }

    #[test]
    fn surlignement_is_useless() {
        let config = setup_test_config();
        assert!(is_empty_or_useless_line(
            "- Votre surlignement Emplacement 1212-1214 | Ajouté le samedi 20 octobre 2018 à 12:55:45",
            &config.parser
        ));
    }

    #[test]
    fn signet_is_useless() {
        let config = setup_test_config();
        assert!(is_empty_or_useless_line(
            "- Votre signet Emplacement 5527 | Ajouté le vendredi 16 novembre 2018 à 11:51:19",
            &config.parser
        ));
    }

    #[test]
    fn note_is_useless() {
        let config = setup_test_config();
        assert!(is_empty_or_useless_line(
            "- Votre note Emplacement 3752 | Ajoutée le vendredi 16 novembre 2018 à 13:51:19",
            &config.parser
        ));
    }

    #[test]
    fn empty_is_useless() {
        let config = setup_test_config();
        assert!(is_empty_or_useless_line("", &config.parser));
    }

    #[test]
    fn highlight_is_useful() {
        let config = setup_test_config();
        assert!(!is_empty_or_useless_line(
            "A standard fake highlight",
            &config.parser
        ));
    }

    #[test]
    fn test_parse_note() {
        let config = setup_test_config();
        let fake_note = vec![
            "A fake title (Last, First)".to_string(),
            "- Votre surlignement Emplacement 3592-3592 | Ajouté le mardi 6 novembre 2018 à 08:50:39"
                .to_string(),
            "".to_string(),
            "This is a fake highlight.".to_string(),
        ];

        if let Some(read_note) = parse_note(&fake_note, &config.parser) {
            assert_eq!(read_note.title, "A fake title (Last, First)");
            assert_eq!(read_note.tidied_note, "This is a fake highlight.");
        } else {
            panic!("The parsed note should not be empty");
        }
    }
}
