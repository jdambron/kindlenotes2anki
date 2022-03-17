use crate::app_config::AppConfig;
use crate::Note;
use std::path::PathBuf;

pub fn parse_clippings(filename: PathBuf) -> Vec<Note> {
    let separator = "==========\r\n";
    let content = std::fs::read_to_string(filename).unwrap();
    content.split(separator).filter_map(parse_note).collect()
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
        .map(|x| x.trim().trim_start_matches('\u{feff}'))
        .collect()
}

fn tidy_note(note: &str) -> String {
    note.lines()
        .skip(1)
        .filter_map(|l| {
            if is_useless_line(l) {
                None
            } else {
                Some((*l).to_string())
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn is_useless_line(line: &str) -> bool {
    let highlight = AppConfig::get::<String>("parser.highlight").unwrap();
    let bookmark = AppConfig::get::<String>("parser.bookmark").unwrap();
    let note = AppConfig::get::<String>("parser.note").unwrap();
    line.starts_with(&highlight)
        || line.starts_with(&bookmark)
        || line.starts_with(&note)
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
}
