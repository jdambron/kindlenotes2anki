use anyhow::{Ok, Result, bail};
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

async fn add_notes(notes: Vec<crate::Note>) -> Result<()> {
    let notes_count: usize = notes.len();
    let mut req = AddNotes {
        action: "addNotes".to_string(),
        version: 6,
        params: Notes { notes: Vec::new() },
    };
    let notes_params = notes.into_iter().map(fill_note_api_params).collect();
    req.params.notes = notes_params;
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
    Note {
        deck_name: DECK_NAME.to_string(),
        model_name: MODEL_NAME.to_string(),
        fields,
        options: Options {
            allow_duplicate: true,
            duplicate_scope: "deck".to_string(),
        },
    }
}

pub async fn write_notes_ankiconnect(notes: Vec<crate::Note>) -> Result<()> {
    add_notes(notes).await?; // .await the future from add_notes
    Ok(())
}
