use crate::app_config::AppConfig;
use crate::Note;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

const SEPARATOR: &str = "==========";

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

    let title = lines[0].trim().to_string();
    let tidied_note: String = lines[1..]
        .iter()
        .filter(|l| !is_useless_line(l))
        .cloned()
        .collect::<Vec<String>>()
        .join("\n");

    if title.is_empty() || tidied_note.is_empty() {
        None
    } else {
        Some(Note { title, tidied_note })
    }
}

fn is_useless_line(line: &str) -> bool {
    line.starts_with(HIGHLIGHT_VALUE.as_str())
        || line.starts_with(BOOKMARK_VALUE.as_str())
        || line.starts_with(NOTE_VALUE.as_str())
        || line.is_empty()
}

lazy_static::lazy_static! {
    static ref HIGHLIGHT_VALUE: String = AppConfig::get::<String>("parser.highlight").unwrap();
    static ref BOOKMARK_VALUE: String = AppConfig::get::<String>("parser.bookmark").unwrap();
    static ref NOTE_VALUE: String = AppConfig::get::<String>("parser.note").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn setup() {
        crate::app_config::AppConfig::init(Some(PathBuf::from("src/resources/test_config.toml")))
            .unwrap();
    }

    #[test]
    #[serial]
    fn surlignement_is_useless() {
        setup();
        assert!(is_useless_line("- Votre surlignement Emplacement 1212-1214 | Ajouté le samedi 20 octobre 2018 à 12:55:45"));
    }

    #[test]
    #[serial]
    fn signet_is_useless() {
        setup();
        assert!(is_useless_line(
            "- Votre signet Emplacement 5527 | Ajouté le vendredi 16 novembre 2018 à 11:51:19"
        ));
    }

    #[test]
    #[serial]
    fn note_is_useless() {
        setup();
        assert!(is_useless_line(
            "- Votre note Emplacement 3752 | Ajoutée le vendredi 16 novembre 2018 à 13:51:19"
        ));
    }

    #[test]
    #[serial]
    fn empty_is_useless() {
        setup();
        assert!(is_useless_line(""));
    }

    #[test]
    #[serial]
    fn highlight_is_useful() {
        setup();
        assert!(!is_useless_line("A standard fake highlight"));
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
