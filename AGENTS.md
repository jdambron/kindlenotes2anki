# AGENTS.md

Single-binary Rust CLI (`edition = "2024"`). Parses Kindle `My Clippings.txt` and either writes CSV or POSTs to AnkiConnect.

## Verify

CI (`.github/workflows/ci.yml`) is the source of truth:

```
cargo fmt --all -- --check
cargo clippy -- -D clippy::all -D clippy::pedantic -D warnings
cargo test
```

After dependency changes: `cargo deny check`.

No `tests/` dir. Tests live in `#[cfg(test)]` modules next to the code. Fixtures: `src/resources/fixtures/`. AnkiConnect tests spawn a local TCP mock; Anki is not required.

`cargo test <name>` runs a single test.

## Layout

| Path | Role |
| --- | --- |
| `src/main.rs` | CLI (`clap`): parse → CSV or AnkiConnect |
| `src/my_clippings_parser.rs` | `==========`-separated Kindle format |
| `src/csv_writer.rs` | two-column CSV (title, note) |
| `src/connect.rs` | AnkiConnect `addNotes` v6 |
| `src/app_config.rs` | TOML config + built-in defaults |
| `src/resources/english_config.toml` | English clippings + English Anki note type |
| `src/resources/default_config.toml` | documented snapshot of Rust `Default` — **not loaded at runtime** |

`AppConfig::new(None)` uses the `Default` impls, not the TOML file.

## Defaults agents guess wrong

- Built-in defaults are **French**: parser prefixes (`- Votre signet` / `- Votre surlignement` / `- Votre note`) and Anki model `Basique` with `Recto` / `Verso`.
- Omitted `[parser]` or `[anki]` sections keep those defaults. If `[parser]` is present, `bookmark` / `highlight` / `note` are required; only `ignored` is optional.
- CSV goes to stdout; the success line (`Exported N notes`) goes to **stderr**. Do not log to stdout in CSV mode.
- AnkiConnect always sends `allowDuplicate: true` (many cards share the book title on the front). Do not “fix” this.
- `ureq` is built with `default-features = false` + `json` only (local HTTP, no TLS).

`My Clippings.txt` and `*.csv` are gitignored.
