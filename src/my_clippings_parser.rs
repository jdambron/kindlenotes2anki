use crate::Note;
use crate::app_config::AppConfig;
use regex::{RegexSet, escape};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::OnceLock;

const SEPARATOR: &str = "==========";
static USELESS_REGEX_SET: OnceLock<RegexSet> = OnceLock::new();

pub fn parse_clippings(filename: PathBuf) -> Result<Vec<Note>, std::io::Error> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut notes = Vec::with_capacity(100); // preallocate some space
    let mut current_note = Vec::with_capacity(10); // preallocate some space

    for line in reader.lines() {
        let line = line?;
        if line.starts_with(SEPARATOR) {
            if !current_note.is_empty() {
                if let Some(note) = parse_note(&current_note) {
                    notes.push(note);
                }
                current_note.clear();
            }
        } else {
            current_note.push(line);
        }
    }

    // Handle the last note
    if !current_note.is_empty() {
        if let Some(note) = parse_note(&current_note) {
            notes.push(note);
        }
    }

    Ok(notes)
}

fn parse_note(lines: &[String]) -> Option<Note> {
    if lines.is_empty() {
        return None;
    }

    let title = lines[0].trim();
    if is_empty_or_useless_line(title) {
        return None;
    }

    let mut tidied_note = String::with_capacity(lines.len() * 20);
    let mut is_first_line = true;

    for line in &lines[1..] {
        if !is_empty_or_useless_line(line) {
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

fn is_empty_or_useless_line(line: &str) -> bool {
    if line.is_empty() {
        return true;
    }

    let regex_set = USELESS_REGEX_SET.get_or_init(|| {
        let patterns = [
            format!("^{}", escape(highlight_value())),
            format!("^{}", escape(bookmark_value())),
            format!("^{}", escape(note_value())),
        ];
        RegexSet::new(patterns).expect("Invalid regex pattern")
    });

    regex_set.is_match(line)
}

fn highlight_value() -> &'static String {
    static HIGHLIGHT_VALUE: OnceLock<String> = OnceLock::new();
    HIGHLIGHT_VALUE.get_or_init(|| {
        AppConfig::get::<String>("parser.highlight")
            .expect("Failed to load 'highlight' value from config. Please check your config file.")
    })
}

fn bookmark_value() -> &'static String {
    static BOOKMARK_VALUE: OnceLock<String> = OnceLock::new();
    BOOKMARK_VALUE.get_or_init(|| {
        AppConfig::get::<String>("parser.bookmark")
            .expect("Failed to load 'bookmark' value from config. Please check your config file.")
    })
}

fn note_value() -> &'static String {
    static NOTE_VALUE: OnceLock<String> = OnceLock::new();
    NOTE_VALUE.get_or_init(|| {
        AppConfig::get::<String>("parser.note")
            .expect("Failed to load 'note' value from config. Please check your config file.")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::tempdir;

    fn setup() {
        crate::app_config::AppConfig::init(Some(PathBuf::from("src/resources/test_config.toml")))
            .unwrap();
    }

    #[test]
    fn test_parse_clippings_empty() {
        let temp_dir = tempdir().expect("Failed to create temporary directory.");
        let file_path = temp_dir.path().join("test_clippings.txt");
        std::fs::File::create(&file_path).expect("Failed to create temporary file.");

        let notes = parse_clippings(file_path).expect("Failed to parse temporary file.");
        assert!(notes.is_empty());
    }

    #[test]
    #[serial]
    fn surlignement_is_useless() {
        setup();
        assert!(is_empty_or_useless_line(
            "- Votre surlignement Emplacement 1212-1214 | Ajouté le samedi 20 octobre 2018 à 12:55:45"
        ));
    }

    #[test]
    #[serial]
    fn signet_is_useless() {
        setup();
        assert!(is_empty_or_useless_line(
            "- Votre signet Emplacement 5527 | Ajouté le vendredi 16 novembre 2018 à 11:51:19"
        ));
    }

    #[test]
    #[serial]
    fn note_is_useless() {
        setup();
        assert!(is_empty_or_useless_line(
            "- Votre note Emplacement 3752 | Ajoutée le vendredi 16 novembre 2018 à 13:51:19"
        ));
    }

    #[test]
    #[serial]
    fn empty_is_useless() {
        setup();
        assert!(is_empty_or_useless_line(""));
    }

    #[test]
    #[serial]
    fn highlight_is_useful() {
        setup();
        assert!(!is_empty_or_useless_line("A standard fake highlight"));
    }

    #[test]
    #[serial]
    fn test_parse_note() {
        setup();
        let fake_note = vec![
            "A fake title (Last, First)".to_string(),
            "- Votre surlignement Emplacement 3592-3592 | Ajouté le mardi 6 novembre 2018 à 08:50:39".to_string(),
            "".to_string(),
            "This is a fake highlight.".to_string(),
        ];
        if let Some(read_note) = parse_note(&fake_note) {
            assert_eq!(read_note.title, "A fake title (Last, First)");
            assert_eq!(read_note.tidied_note, "This is a fake highlight.");
        } else {
            panic!("The parsed note should not be empty");
        }
    }
}
