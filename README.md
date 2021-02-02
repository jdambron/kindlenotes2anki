# kindlenotes2anki

![Continuous integration](https://github.com/jdambron/kindlenotes2anki/workflows/Continuous%20integration/badge.svg)
![Rust](https://github.com/jdambron/kindlenotes2anki/workflows/Rust/badge.svg)

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
USAGE:
    kindlenotes2anki.exe [FLAGS] [OPTIONS] <clippings>

FLAGS:
    -c, --connect    Use AnkiConnect, if not provided will generate a CSV output
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --config <config>    The path to a config file, if not provided will use defaults

ARGS:
    <clippings>    The path to the clippings txt file to read
```

## Configuration

You can adapt the configuration to your clippings file language customzing the configuration file.
