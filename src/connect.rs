use crate::app_config::AnkiConfig;
use crate::note::Note as AppNote;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Duration;

const DUPLICATE_SCOPE: &str = "deck";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Deserialize)]
struct ApiResponse {
    result: Option<Vec<Option<usize>>>,
    error: Option<String>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct AddNotes<'a> {
    action: &'a str,
    version: usize,
    params: Notes<'a>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct Notes<'a> {
    notes: Vec<Note<'a>>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct Note<'a> {
    deck_name: &'a str,
    model_name: &'a str,
    fields: BTreeMap<&'a str, &'a str>,
    options: Options<'a>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct Options<'a> {
    allow_duplicate: bool,
    duplicate_scope: &'a str,
}

pub fn add_notes(notes: &[AppNote], config: &AnkiConfig) -> Result<usize> {
    if notes.is_empty() {
        return Ok(0);
    }
    let notes_count = notes.len();
    let req = build_add_notes_request(notes, config);
    let agent_config = ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .build();
    let agent = ureq::Agent::new_with_config(agent_config);
    let mut response = match agent.post(&config.url).send_json(&req) {
        Ok(response) => response,
        Err(ureq::Error::StatusCode(code)) => bail!(
            "AnkiConnect at {} returned HTTP {code}. Check that Anki is idle (not syncing) and retry.",
            config.url
        ),
        Err(err) => {
            return Err(err).with_context(|| {
                format!(
                    "Failed to connect to AnkiConnect at {}. Is Anki running with the AnkiConnect add-on?",
                    config.url
                )
            });
        }
    };
    let parsed: ApiResponse = response
        .body_mut()
        .read_json()
        .context("Failed to parse AnkiConnect response")?;

    if let Some(error) = parsed.error {
        bail!("AnkiConnect error: {error}");
    }
    let Some(result) = parsed.result else {
        bail!("Unexpected AnkiConnect response: missing result");
    };
    let created = result.into_iter().flatten().count();
    if created == notes_count {
        Ok(created)
    } else {
        bail!("Some notes could not be created ({created}/{notes_count} succeeded)");
    }
}

fn build_add_notes_request<'a>(notes: &'a [AppNote], config: &'a AnkiConfig) -> AddNotes<'a> {
    AddNotes {
        action: "addNotes",
        version: 6,
        params: Notes {
            notes: notes
                .iter()
                .map(|note| fill_note_api_params(note, config))
                .collect(),
        },
    }
}

fn fill_note_api_params<'a>(note: &'a AppNote, config: &'a AnkiConfig) -> Note<'a> {
    let mut fields = BTreeMap::new();
    fields.insert(config.front_field.as_str(), note.title.as_str());
    fields.insert(config.back_field.as_str(), note.tidied_note.as_str());
    Note {
        deck_name: &config.deck,
        model_name: &config.model,
        fields,
        options: Options {
            allow_duplicate: true,
            duplicate_scope: DUPLICATE_SCOPE,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::Note as AppNote;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    fn sample_notes() -> Vec<AppNote> {
        vec![
            AppNote {
                title: "Book A".to_owned(),
                tidied_note: "Highlight 1".to_owned(),
            },
            AppNote {
                title: "Book A".to_owned(),
                tidied_note: "Highlight 2".to_owned(),
            },
        ]
    }

    fn read_http_request(stream: &mut impl Read) -> String {
        let mut buf = Vec::new();
        let mut chunk = [0_u8; 4096];
        loop {
            let n = stream.read(&mut chunk).unwrap_or(0);
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
            if let Some(header_end) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = std::str::from_utf8(&buf[..header_end]).unwrap_or("");
                let content_length = headers.lines().find_map(|line| {
                    let line = line.trim();
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|v| v.trim().parse::<usize>().ok())
                });
                if let Some(len) = content_length {
                    let total = header_end + 4 + len;
                    while buf.len() < total {
                        let n = stream.read(&mut chunk).unwrap_or(0);
                        if n == 0 {
                            break;
                        }
                        buf.extend_from_slice(&chunk[..n]);
                    }
                }
                break;
            }
        }
        String::from_utf8_lossy(&buf).into_owned()
    }

    fn spawn_server(
        status: &'static str,
        body: &'static str,
    ) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let request = read_http_request(&mut stream);
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
            request
        });
        (format!("http://{addr}"), handle)
    }

    fn spawn_json_server(response_body: &'static str) -> (String, thread::JoinHandle<String>) {
        spawn_server("200 OK", response_body)
    }

    #[test]
    fn build_request_uses_config_and_allows_duplicates() {
        let config = AnkiConfig {
            deck: "Clippings".to_owned(),
            model: "Basic".to_owned(),
            front_field: "Front".to_owned(),
            back_field: "Back".to_owned(),
            url: "http://localhost:8765".to_owned(),
        };
        let notes = sample_notes();
        let req = build_add_notes_request(&notes, &config);

        assert_eq!(req.action, "addNotes");
        assert_eq!(req.version, 6);
        assert_eq!(req.params.notes.len(), 2);
        assert_eq!(req.params.notes[0].deck_name, "Clippings");
        assert_eq!(req.params.notes[0].model_name, "Basic");
        assert_eq!(
            req.params.notes[0].fields.get("Front").copied(),
            Some("Book A")
        );
        assert_eq!(
            req.params.notes[0].fields.get("Back").copied(),
            Some("Highlight 1")
        );
        assert_eq!(
            req.params.notes[1].fields.get("Back").copied(),
            Some("Highlight 2")
        );
        assert!(req.params.notes[0].options.allow_duplicate);
        assert_eq!(req.params.notes[0].options.duplicate_scope, "deck");
    }

    #[test]
    fn add_notes_posts_to_configured_url() {
        let (url, server) = spawn_json_server(r#"{"result":[1,2],"error":null}"#);
        let config = AnkiConfig {
            url,
            ..AnkiConfig::default()
        };
        let created = add_notes(&sample_notes(), &config).unwrap();
        assert_eq!(created, 2);
        let request = server.join().unwrap();
        assert!(request.starts_with("POST /"));
        assert!(request.contains("addNotes"));
        assert!(request.contains("allowDuplicate"));
    }

    #[test]
    fn add_notes_reports_partial_failure() {
        let (url, server) = spawn_json_server(r#"{"result":[1,null],"error":null}"#);
        let config = AnkiConfig {
            url,
            ..AnkiConfig::default()
        };
        let err = add_notes(&sample_notes(), &config).unwrap_err();
        assert!(
            err.to_string().contains("1/2 succeeded"),
            "unexpected error: {err}"
        );
        server.join().unwrap();
    }

    #[test]
    fn add_notes_with_no_notes_makes_no_http_call() {
        // Connection would be refused if any HTTP call were attempted
        let config = AnkiConfig {
            url: "http://127.0.0.1:1".to_owned(),
            ..AnkiConfig::default()
        };
        let created = add_notes(&[], &config).unwrap();
        assert_eq!(created, 0);
    }

    #[test]
    fn add_notes_reports_api_error_with_null_result() {
        let (url, server) = spawn_json_server(r#"{"result":null,"error":"database is locked"}"#);
        let config = AnkiConfig {
            url,
            ..AnkiConfig::default()
        };
        let err = add_notes(&sample_notes(), &config).unwrap_err();
        assert!(
            err.to_string().contains("database is locked"),
            "unexpected error: {err}"
        );
        server.join().unwrap();
    }

    #[test]
    fn add_notes_reports_http_status_error() {
        let (url, server) = spawn_server(
            "500 Internal Server Error",
            r#"{"result":null,"error":"boom"}"#,
        );
        let config = AnkiConfig {
            url,
            ..AnkiConfig::default()
        };
        let err = add_notes(&sample_notes(), &config).unwrap_err();
        assert!(
            err.to_string().contains("HTTP 500"),
            "unexpected error: {err}"
        );
        server.join().unwrap();
    }
}
