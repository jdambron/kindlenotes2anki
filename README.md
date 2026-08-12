# kindlenotes2anki

[![Continuous integration](https://github.com/jdambron/kindlenotes2anki/actions/workflows/ci.yml/badge.svg)](https://github.com/jdambron/kindlenotes2anki/actions/workflows/ci.yml)

A tool to import Kindle clippings (`My Clippings.txt`) into [Anki](https://apps.ankiweb.net/).

There are 2 modes:

1. Generate CSV output that can be imported into Anki (default)
2. Direct import using [AnkiConnect](https://foosoft.net/projects/anki-connect/)

To use direct import, install the AnkiConnect add-on and launch Anki first.

## Disclaimer

This is mostly a project to play around with [Rust](https://www.rust-lang.org/) and learn it on something real.
So there are probably lots of things that can be improved.

## Build the project

```shell
cargo build --release
```

## Usage

```text
A tool to import kindle clippings file to Anki

Usage: kindlenotes2anki [OPTIONS] <CLIPPINGS>

Arguments:
  <CLIPPINGS>  The path to the clippings txt file to read

Options:
  -u, --use-anki-connect     Use AnkiConnect, if not provided will generate a CSV output
  -o, --output <PATH>        Write CSV to this file instead of stdout (ignored with --use-anki-connect)
      --config <CONFIG>      The path to a config file, if not provided will use defaults
  -h, --help                 Print help
  -V, --version              Print version
```

### Examples

```shell
# CSV to stdout (import manually in Anki)
kindlenotes2anki "My Clippings.txt" > notes.csv

# CSV to a file
kindlenotes2anki -o notes.csv "My Clippings.txt"

# Direct import via AnkiConnect (French Anki defaults)
kindlenotes2anki -u "My Clippings.txt"

# English clippings + English Anki note type
kindlenotes2anki -u --config src/resources/english_config.toml "My Clippings.txt"
```

On success, a short summary is printed to stderr (for example `Exported 42 notes`), so it does not mix with CSV on stdout.

## Configuration

Defaults target a **French** `My Clippings.txt` and the French Anki note type (`Basique` with `Recto` / `Verso`).
Customize language and Anki settings with a TOML config file:

```toml
[parser]
bookmark = "- Votre signet"
highlight = "- Votre surlignement"
note = "- Votre note"
# Note content is cut at the first ignored marker; cards left empty are skipped
ignored = ["<Vous avez atteint la limite maximale", "<You have reached the clipping limit"]

[anki]
deck = "Kindle"
model = "Basique"
front_field = "Recto"
back_field = "Verso"
url = "http://localhost:8765"
```

For English clippings and the default English Anki note type, see `src/resources/english_config.toml`:

```toml
[parser]
bookmark = "- Your Bookmark"
highlight = "- Your Highlight"
note = "- Your Note"

[anki]
deck = "Kindle"
model = "Basic"
front_field = "Front"
back_field = "Back"
url = "http://localhost:8765"
```

Sections are optional: omitted `[parser]` or `[anki]` values keep the built-in defaults.
The built-in `ignored` patterns filter the notices Kindle injects when a book's clipping limit is reached.

The parser also transparently handles UTF-8 BOMs (including the per-entry BOMs some Kindle firmware inserts after each separator) and CRLF line endings.

Duplicate notes are always allowed (`allowDuplicate: true`) because many highlights share the same book title on the front of the card.
