use crate::Note;
use std::path::PathBuf;

const BOOKMARK: &str = "- Votre signet";
const HIGHLIGHT: &str = "- Votre surlignement";
const NOTE: &str = "- Votre note";

pub fn parse_clippings(filename: PathBuf) -> Vec<Note> {
    let separator = "==========\r\n";
    let content = std::fs::read_to_string(filename).unwrap();
    content
        .split(separator)
        .filter_map(|n| parse_note(n))
        .collect()
}

fn parse_note(note: &str) -> Option<Note> {
    let lines: Vec<&str> = note.lines().collect();
    let title: String = lines
        .iter()
        .take(1)
        .map(|x| x.trim().trim_start_matches("\u{feff}"))
        .collect();
    let tidied_note: String = lines
        .iter()
        .skip(1)
        .filter_map(|l| {
            if is_useless_line(l) {
                None
            } else {
                Some((*l).to_string())
            }
        })
        .collect::<Vec<String>>()
        .join("\n");
    if title.is_empty() || tidied_note.is_empty() {
        None
    } else {
        Some(Note { title, tidied_note })
    }
}

fn is_useless_line(line: &str) -> bool {
    line.starts_with(HIGHLIGHT)
        || line.starts_with(BOOKMARK)
        || line.starts_with(NOTE)
        || line.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surlignement_is_useless() {
        assert!(is_useless_line("- Votre surlignement Emplacement 1212-1214 | Ajouté le samedi 20 octobre 2018 à 12:55:45"));
    }

    #[test]
    fn signet_is_useless() {
        assert!(is_useless_line(
            "- Votre signet Emplacement 5527 | Ajouté le vendredi 16 novembre 2018 à 11:51:19"
        ));
    }

    #[test]
    fn note_is_useless() {
        assert!(is_useless_line(
            "- Votre note Emplacement 3752 | Ajoutée le vendredi 16 novembre 2018 à 13:51:19"
        ));
    }

    #[test]
    fn empty_is_useless() {
        assert!(is_useless_line(""));
    }

    #[test]
    fn highlight_is_useful() {
        assert!(!is_useless_line("A standard fake highlight"));
    }

    #[test]
    fn test_parse_note() {
        let fake_note = "A fake title (Last, First)\n- Votre surlignement Emplacement 3592-3592 | Ajouté le mardi 6 novembre 2018 à 08:50:39\n\nThis is a fake highlight.\n";
        if let Some(read_note) = parse_note(fake_note) {
            assert_eq!(read_note.title, "A fake title (Last, First)");
            assert_eq!(read_note.tidied_note, "This is a fake highlight.");
        } else {
            panic!("The parsed note should not be empty");
        }
    }
}
