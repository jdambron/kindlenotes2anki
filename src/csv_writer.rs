use crate::note::Note;
use anyhow::Result;
use std::io::Write;

pub fn write_csv(notes: &[Note], writer: impl Write) -> Result<usize> {
    let mut wtr = csv::Writer::from_writer(writer);
    for note in notes {
        wtr.write_record([&note.title, &note.tidied_note])?;
    }
    wtr.flush()?;
    Ok(notes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::Note;

    #[test]
    fn writes_title_and_note_rows() {
        let notes = vec![
            Note {
                title: "Book".to_owned(),
                tidied_note: "Line 1\nLine 2".to_owned(),
            },
            Note {
                title: "Other".to_owned(),
                tidied_note: "Highlight".to_owned(),
            },
        ];
        let mut buf = Vec::new();
        let count = write_csv(&notes, &mut buf).unwrap();
        assert_eq!(count, 2);
        let csv = String::from_utf8(buf).unwrap();
        assert_eq!(csv, "Book,\"Line 1\nLine 2\"\nOther,Highlight\n");
    }

    #[test]
    fn empty_notes_writes_nothing() {
        let mut buf = Vec::new();
        let count = write_csv(&[], &mut buf).unwrap();
        assert_eq!(count, 0);
        assert!(buf.is_empty());
    }
}
