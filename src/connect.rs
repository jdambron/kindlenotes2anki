use serde::Deserialize;
use serde_json::json;
use std::error::Error;

const DECK_NAME: &str = "Kindle";
const MODEL_NAME: &str = "Basique";

#[derive(Deserialize)]
struct ApiResponse {
    result: Option<usize>,
    error: Option<String>,
}

#[tokio::main]
async fn add_note(recto: &str, verso: &str) -> Result<(), Box<dyn Error>> {
    let req = json!({
        "action": "addNote",
        "version": 6,
        "params": {
            "note": {
                "deckName": DECK_NAME,
                "modelName": MODEL_NAME,
                "fields": {
                    "Recto": recto,
                    "Verso": verso,
                },
                "options": {
                    "allowDuplicate": true,
                    "duplicateScope": "deck"
                },
            },
        },
    });
    let client = reqwest::Client::new();
    let res = client
        .post("http://localhost:8765")
        .json(&req)
        .send()
        .await?
        .json::<ApiResponse>()
        .await?;
    match res.result {
        Some(_) => Ok(()),
        None => match res.error {
            Some(error) => bail!(error),
            None => bail!("Unknown error"),
        },
    }
}

pub fn write_notes_ankiconnect(notes: Vec<crate::Note>) {
    for note in notes {
        add_note(&note.title, &note.tidied_note).unwrap();
    }
}
