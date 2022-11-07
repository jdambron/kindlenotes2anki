# kindlenotes2anki

[![Continuous integration](https://github.com/jdambron/kindlenotes2anki/actions/workflows/ci.yml/badge.svg)](https://github.com/jdambron/kindlenotes2anki/actions/workflows/ci.yml)

A tool to import Kindle clippings file to [Anki](https://apps.ankiweb.net/)

There are 2 modes :

1. Generate a CSV output than can be imported into Anki (default)
1. Direct import using [AnkiConnect](https://foosoft.net/projects/anki-connect/)

⚠️ To be able to use the direct import, you must first install AnkiConnect and launch Anki.

## Disclaimer

This is mostly a project to play around with [Rust](https://www.rust-lang.org/) and learn it on something real.
So there are probably lots of things that can be improved.

## Build the project

```Shell
cargo build --release
```

## Usage

```
A tool to import kindle clippings file to Anki

Usage: kindlenotes2anki.exe [OPTIONS] <CLIPPINGS>

Arguments:
  <CLIPPINGS>  The path to the clippings txt file to read

Options:
  -c, --connect          Use AnkiConnect, if not provided will generate a CSV output
      --config <CONFIG>  The path to a config file, if not provided will use defaults
  -h, --help             Print help information
  -V, --version          Print version information
```

## Configuration

You can adapt the configuration to your clippings file language customzing the configuration file.
