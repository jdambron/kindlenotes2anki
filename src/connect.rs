use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

const DECK_NAME: &str = "Kindle";
const MODEL_NAME: &str = "Basique";

#[derive(Deserialize)]
struct ApiResponse {
    result: Vec<Option<usize>>,
    error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AddNotes {
    action: String,
    version: usize,
    params: Notes,
}

#[derive(Serialize)]
#[serde(rename = "camelCase")]
struct Notes {
    notes: Vec<Note>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Note {
    deck_name: String,
    model_name: String,
    fields: Fields,
    options: Options,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Fields {
    recto: String,
    verso: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Options {
    allow_duplicate: bool,
    duplicate_scope: String,
}

#[tokio::main]
async fn add_notes(notes: Vec<crate::Note>) -> Result<()> {
    let notes_count: usize = notes.len();
    let mut req = AddNotes {
        action: "addNotes".to_string(),
        version: 6,
        params: Notes { notes: Vec::new() },
    };
    for note in notes {
        let new_note = fill_note_api_params(note);
        req.params.notes.push(new_note);
    }
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:8765")
        .json(&req)
        .send()
        .await?
        .json::<ApiResponse>()
        .await?;
    match response.error {
        Some(error) => bail!(error),
        None => {
            if response.result.into_iter().flatten().count() == notes_count {
                Ok(())
            } else {
                bail!("Some notes could not be created");
            }
        }
    }
}

fn fill_note_api_params(note: crate::Note) -> Note {
    let fields = Fields {
        recto: note.title,
        verso: note.tidied_note,
    };
    let options = Options {
        allow_duplicate: true,
        duplicate_scope: "deck".to_string(),
    };
    Note {
        deck_name: DECK_NAME.to_string(),
        model_name: MODEL_NAME.to_string(),
        fields,
        options,
    }
}

pub fn write_notes_ankiconnect(notes: Vec<crate::Note>) {
    add_notes(notes).unwrap();
}
