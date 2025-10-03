use crate::Note;
use crate::app_config::AppConfig;
use regex::{RegexSet, escape};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const SEPARATOR: &str = "==========";

pub fn parse_clippings(filename: PathBuf, config: &AppConfig) -> Result<Vec<Note>, std::io::Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut notes = Vec::with_capacity(100);
    let mut current_note = Vec::with_capacity(10);

    // Create the regex set from the passed-in config
    let useless_regex_set = RegexSet::new(&[
        format!("^{}", escape(&config.parser.highlight)),
        format!("^{}", escape(&config.parser.bookmark)),
        format!("^{}", escape(&config.parser.note)),
    ])
    .expect("Invalid regex pattern");

    for line in reader.lines() {
        let line = line?;
        if line.starts_with(SEPARATOR) {
            if !current_note.is_empty() {
                // Pass the regex set to the parsing function
                if let Some(note) = parse_note(&current_note, &useless_regex_set) {
                    notes.push(note);
                }
                current_note.clear();
            }
        } else {
            current_note.push(line);
        }
    }

    if !current_note.is_empty()
        && let Some(note) = parse_note(&current_note, &useless_regex_set)
    {
        notes.push(note);
    }

    Ok(notes)
}

fn parse_note(lines: &[String], useless_regex_set: &RegexSet) -> Option<Note> {
    if lines.is_empty() {
        return None;
    }

    let title = lines[0].trim();
    if is_empty_or_useless_line(title, useless_regex_set) {
        return None;
    }

    let mut tidied_note = String::with_capacity(lines.len() * 20);
    let mut is_first_line = true;

    for line in &lines[1..] {
        if !is_empty_or_useless_line(line, useless_regex_set) {
            if !is_first_line {
                tidied_note.push('\n');
            }
            tidied_note.push_str(line);
            is_first_line = false;
        }
    }

    if tidied_note.is_empty() {
        None
    } else {
        Some(Note {
            title: title.to_owned(),
            tidied_note,
        })
    }
}

fn is_empty_or_useless_line(line: &str, useless_regex_set: &RegexSet) -> bool {
    if line.is_empty() {
        return true;
    }
    useless_regex_set.is_match(line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::AppConfig;
    use regex::RegexSet;
    use std::path::PathBuf;
    use tempfile::tempdir;

    // Helper function to load the test configuration for most tests.
    fn setup_test_config() -> AppConfig {
        AppConfig::new(Some(PathBuf::from("src/resources/test_config.toml"))).unwrap()
    }

    // Helper to create the RegexSet based on a given config.
    fn create_useless_regex_set(config: &AppConfig) -> RegexSet {
        RegexSet::new(&[
            format!("^{}", escape(&config.parser.highlight)),
            format!("^{}", escape(&config.parser.bookmark)),
            format!("^{}", escape(&config.parser.note)),
        ])
        .unwrap()
    }

    #[test]
    fn test_parse_clippings_empty() {
        let config = setup_test_config(); // Load config for the test
        let temp_dir = tempdir().expect("Failed to create temporary directory.");
        let file_path = temp_dir.path().join("test_clippings.txt");
        std::fs::File::create(&file_path).expect("Failed to create temporary file.");

        // Pass the config to the function
        let notes = parse_clippings(file_path, &config).expect("Failed to parse temporary file.");
        assert!(notes.is_empty());
    }

    #[test]
    fn surlignement_is_useless() {
        let config = setup_test_config();
        let regex_set = create_useless_regex_set(&config);
        assert!(is_empty_or_useless_line(
            "- Votre surlignement Emplacement 1212-1214 | Ajouté le samedi 20 octobre 2018 à 12:55:45",
            &regex_set
        ));
    }

    #[test]
    fn signet_is_useless() {
        let config = setup_test_config();
        let regex_set = create_useless_regex_set(&config);
        assert!(is_empty_or_useless_line(
            "- Votre signet Emplacement 5527 | Ajouté le vendredi 16 novembre 2018 à 11:51:19",
            &regex_set
        ));
    }

    #[test]
    fn note_is_useless() {
        let config = setup_test_config();
        let regex_set = create_useless_regex_set(&config);
        assert!(is_empty_or_useless_line(
            "- Votre note Emplacement 3752 | Ajoutée le vendredi 16 novembre 2018 à 13:51:19",
            &regex_set
        ));
    }

    #[test]
    fn empty_is_useless() {
        let config = setup_test_config();
        let regex_set = create_useless_regex_set(&config);
        assert!(is_empty_or_useless_line("", &regex_set));
    }

    #[test]
    fn highlight_is_useful() {
        let config = setup_test_config();
        let regex_set = create_useless_regex_set(&config);
        assert!(!is_empty_or_useless_line(
            "A standard fake highlight",
            &regex_set
        ));
    }

    #[test]
    fn test_parse_note() {
        let config = setup_test_config();
        let regex_set = create_useless_regex_set(&config);
        let fake_note = vec![
            "A fake title (Last, First)".to_string(),
            "- Votre surlignement Emplacement 3592-3592 | Ajouté le mardi 6 novembre 2018 à 08:50:39"
                .to_string(),
            "".to_string(),
            "This is a fake highlight.".to_string(),
        ];

        if let Some(read_note) = parse_note(&fake_note, &regex_set) {
            assert_eq!(read_note.title, "A fake title (Last, First)");
            assert_eq!(read_note.tidied_note, "This is a fake highlight.");
        } else {
            panic!("The parsed note should not be empty");
        }
    }
}
