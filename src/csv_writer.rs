use crate::Note;
use anyhow::Result;
use std::io;

pub fn write_csv(notes: Vec<Note>) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    for note in notes {
        wtr.write_record(&[note.title, note.tidied_note]).unwrap();
    }
    wtr.flush()?;
    Ok(())
}
