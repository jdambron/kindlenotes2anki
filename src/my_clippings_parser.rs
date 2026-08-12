use crate::app_config::{AppConfig, ParserConfig};
use crate::note::Note;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

const SEPARATOR: &str = "==========";

pub fn parse_clippings(filename: &Path, config: &AppConfig) -> Result<Vec<Note>> {
    let file = File::open(filename)
        .with_context(|| format!("Failed to open clippings file: {}", filename.display()))?;
    let reader = BufReader::new(file);
    let mut notes = Vec::with_capacity(100);
    let mut current_note = Vec::with_capacity(10);
    let prefixes = &config.parser;

    for line in reader.lines() {
        let mut line = line?;
        // Some Kindle firmware prepends a BOM to each appended entry, not
        // just at the start of the file; Windows-edited files may be CRLF.
        if line.starts_with('\u{feff}') {
            line.drain(..'\u{feff}'.len_utf8());
        }
        if line.ends_with('\r') {
            line.pop();
        }
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

    let mut tidied_note: String = lines[1..]
        .iter()
        .filter(|l| !is_empty_or_useless_line(l, prefixes))
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    // Kindle clipping-limit notices terminate whatever they were injected
    // into: cut everything from the first ignored marker onward.
    let cut = prefixes
        .ignored
        .iter()
        .filter_map(|pattern| tidied_note.find(pattern))
        .min();
    if let Some(pos) = cut {
        tidied_note.truncate(pos);
    }

    let tidied_note = tidied_note.trim();
    if tidied_note.is_empty() {
        None
    } else {
        Some(Note {
            title: title.to_owned(),
            tidied_note: tidied_note.to_owned(),
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
    use crate::app_config::{AppConfig, ParserConfig};
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn french_config() -> AppConfig {
        AppConfig::default()
    }

    fn english_config() -> AppConfig {
        AppConfig {
            parser: ParserConfig {
                bookmark: "- Your Bookmark".to_owned(),
                highlight: "- Your Highlight".to_owned(),
                note: "- Your Note".to_owned(),
                ..ParserConfig::default()
            },
            ..AppConfig::default()
        }
    }

    fn write_temp(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_parse_clippings_empty() {
        let file = write_temp("");
        let notes = parse_clippings(file.path(), &french_config()).unwrap();
        assert!(notes.is_empty());
    }

    #[test]
    fn surlignement_is_useless() {
        assert!(is_empty_or_useless_line(
            "- Votre surlignement Emplacement 1212-1214 | Ajouté le samedi 20 octobre 2018 à 12:55:45",
            &french_config().parser
        ));
    }

    #[test]
    fn signet_is_useless() {
        assert!(is_empty_or_useless_line(
            "- Votre signet Emplacement 5527 | Ajouté le vendredi 16 novembre 2018 à 11:51:19",
            &french_config().parser
        ));
    }

    #[test]
    fn note_is_useless() {
        assert!(is_empty_or_useless_line(
            "- Votre note Emplacement 3752 | Ajoutée le vendredi 16 novembre 2018 à 13:51:19",
            &french_config().parser
        ));
    }

    #[test]
    fn empty_is_useless() {
        assert!(is_empty_or_useless_line("", &french_config().parser));
    }

    #[test]
    fn highlight_is_useful() {
        assert!(!is_empty_or_useless_line(
            "A standard fake highlight",
            &french_config().parser
        ));
    }

    #[test]
    fn test_parse_note() {
        let fake_note = vec![
            "A fake title (Last, First)".to_string(),
            "- Votre surlignement Emplacement 3592-3592 | Ajouté le mardi 6 novembre 2018 à 08:50:39"
                .to_string(),
            String::new(),
            "This is a fake highlight.".to_string(),
        ];

        let read_note = parse_note(&fake_note, &french_config().parser).unwrap();
        assert_eq!(read_note.title, "A fake title (Last, First)");
        assert_eq!(read_note.tidied_note, "This is a fake highlight.");
    }

    #[test]
    fn parses_french_multi_block_fixture() {
        let content = include_str!("resources/fixtures/sample_fr.txt");
        let file = write_temp(content);
        let notes = parse_clippings(file.path(), &french_config()).unwrap();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].title, "Livre Exemple (Auteur, A)");
        assert_eq!(notes[0].tidied_note, "Premier surlignement.");
        assert_eq!(notes[1].title, "Livre Exemple (Auteur, A)");
        assert_eq!(notes[1].tidied_note, "Deuxième surlignement.");
    }

    #[test]
    fn parses_english_multi_block_fixture() {
        let content = include_str!("resources/fixtures/sample_en.txt");
        let file = write_temp(content);
        let notes = parse_clippings(file.path(), &english_config()).unwrap();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].title, "Example Book (Author, A)");
        assert_eq!(notes[0].tidied_note, "First highlight.");
        assert_eq!(notes[1].title, "Example Book (Author, A)");
        assert_eq!(notes[1].tidied_note, "Second highlight.");
    }

    #[test]
    fn strips_utf8_bom() {
        let content = include_str!("resources/fixtures/sample_bom.txt");
        let file = write_temp(content);
        let notes = parse_clippings(file.path(), &french_config()).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "Livre BOM (Auteur, B)");
        assert_eq!(notes[0].tidied_note, "Surlignement avec BOM.");
    }

    #[test]
    fn skips_bookmark_only_blocks() {
        let content = "\
Livre (Auteur)
- Votre signet Emplacement 10 | Ajouté le lundi 1 janvier 2020 à 10:00:00

==========
";
        let file = write_temp(content);
        let notes = parse_clippings(file.path(), &french_config()).unwrap();
        assert!(notes.is_empty());
    }

    #[test]
    fn strips_bom_from_every_entry() {
        // Some Kindle firmware prepends a BOM to each appended entry
        let content = "\
Livre (Auteur)
- Votre surlignement Emplacement 1 | Ajouté le lundi 1 janvier 2020 à 10:00:00

Premier.
==========
\u{feff}Livre (Auteur)
- Votre surlignement Emplacement 2 | Ajouté le lundi 1 janvier 2020 à 10:00:00

Deuxième.
==========
";
        let file = write_temp(content);
        let notes = parse_clippings(file.path(), &french_config()).unwrap();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].title, "Livre (Auteur)");
        assert_eq!(notes[1].title, "Livre (Auteur)");
    }

    #[test]
    fn handles_crlf_line_endings() {
        let content = "\
Livre (Auteur)\r
- Votre surlignement Emplacement 1 | Ajouté le lundi 1 janvier 2020 à 10:00:00\r
\r
Première ligne.\r
Deuxième ligne.\r
==========\r
";
        let file = write_temp(content);
        let notes = parse_clippings(file.path(), &french_config()).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "Livre (Auteur)");
        assert_eq!(notes[0].tidied_note, "Première ligne.\nDeuxième ligne.");
    }

    #[test]
    fn skips_clipping_limit_junk_cards() {
        let content = "\
Livre (Auteur)
- Votre surlignement Emplacement 1 | Ajouté le lundi 1 janvier 2020 à 10:00:00

<Vous avez atteint la limite maximale d’extraits pour cet élément.>
==========
Livre (Auteur)
- Votre surlignement Emplacement 2 | Ajouté le lundi 1 janvier 2020 à 10:00:00

Surlignement utile.
==========
";
        let file = write_temp(content);
        let notes = parse_clippings(file.path(), &french_config()).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].tidied_note, "Surlignement utile.");
    }

    #[test]
    fn strips_inline_clipping_limit_notice() {
        let content = "\
Livre (Auteur)
- Votre surlignement Emplacement 1 | Ajouté le lundi 1 janvier 2020 à 10:00:00

Début du surlignement tronqué <Vous avez atteint la limite maximale d’extraits pour cet élément.>
==========
";
        let file = write_temp(content);
        let notes = parse_clippings(file.path(), &french_config()).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].tidied_note, "Début du surlignement tronqué");
    }
}
