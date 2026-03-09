use crate::note::Note as AppNote;
use anyhow::{Ok, Result, bail};
use serde::{Deserialize, Serialize};

const DECK_NAME: &str = "Kindle";
const MODEL_NAME: &str = "Basique";
const DUPLICATE_SCOPE: &str = "deck";

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
#[serde(rename_all = "camelCase")]
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

pub async fn add_notes(notes: Vec<AppNote>) -> Result<()> {
    let notes_count: usize = notes.len();
    let req = AddNotes {
        action: "addNotes".to_string(),
        version: 6,
        params: Notes {
            notes: notes.into_iter().map(fill_note_api_params).collect(),
        },
    };
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

fn fill_note_api_params(note: AppNote) -> Note {
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
            duplicate_scope: DUPLICATE_SCOPE.to_string(),
        },
    }
}
