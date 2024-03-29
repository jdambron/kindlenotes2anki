use crate::app_config::AppConfig;
use crate::Note;
use regex::Regex;
use std::path::PathBuf;

const BOOKMARK: &str = "parser.bookmark";
const HIGHLIGHT: &str = "parser.highlight";
const NOTE: &str = "parser.note";
const SEPARATOR: &str = r"==========\r?\n?";

lazy_static::lazy_static! {
    static ref HIGHLIGHT_VALUE: String = AppConfig::get::<String>(HIGHLIGHT).unwrap();
    static ref BOOKMARK_VALUE: String = AppConfig::get::<String>(BOOKMARK).unwrap();
    static ref NOTE_VALUE: String = AppConfig::get::<String>(NOTE).unwrap();
    static ref REGEX_SEPARATOR: Regex = Regex::new(SEPARATOR).unwrap();
}

pub fn parse_clippings(filename: PathBuf) -> Result<Vec<Note>, std::io::Error> {
    let content = std::fs::read_to_string(filename)?;
    let notes = REGEX_SEPARATOR
        .split(&content)
        .filter_map(parse_note)
        .collect();
    Ok(notes)
}

fn parse_note(note: &str) -> Option<Note> {
    let title: String = get_title(note);
    let tidied_note: String = tidy_note(note);
    if title.is_empty() || tidied_note.is_empty() {
        None
    } else {
        Some(Note { title, tidied_note })
    }
}

fn get_title(note: &str) -> String {
    note.lines()
        .take(1)
        .map(str::trim)
        .map(|x| x.trim_start_matches('\u{feff}'))
        .collect()
}

fn tidy_note(note: &str) -> String {
    note.lines()
        .skip(1)
        .filter(|l| !is_useless_line(l))
        .collect::<Vec<&str>>()
        .join("\n")
}

fn is_useless_line(line: &str) -> bool {
    line.starts_with(HIGHLIGHT_VALUE.as_str())
        || line.starts_with(BOOKMARK_VALUE.as_str())
        || line.starts_with(NOTE_VALUE.as_str())
        || line.is_empty()
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
        let fake_note = "A fake title (Last, First)\n- Votre surlignement Emplacement 3592-3592 | Ajouté le mardi 6 novembre 2018 à 08:50:39\n\nThis is a fake highlight.\n";
        if let Some(read_note) = parse_note(fake_note) {
            assert_eq!(read_note.title, "A fake title (Last, First)");
            assert_eq!(read_note.tidied_note, "This is a fake highlight.");
        } else {
            panic!("The parsed note should not be empty");
        }
    }

    #[test]
    #[serial]
    fn get_title_valid_note() {
        let note = "A valid title\nThis is the note content.";
        assert_eq!(get_title(note), "A valid title");
    }

    #[test]
    #[serial]
    fn tidy_note_valid_note() {
        let note = "- Votre surlignement Emplacement 1212-1214 | Ajouté le samedi 20 octobre 2018 à 12:55:45\nThis is the note content.\n- Votre signet Emplacement 5527 | Ajouté le vendredi 16 novembre 2018 à 11:51:19\n";
        assert_eq!(tidy_note(note), "This is the note content.");
    }
}
