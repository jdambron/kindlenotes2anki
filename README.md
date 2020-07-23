# kindlenotes2anki

![Continuous integration](https://github.com/jdambron/kindlenotes2anki/workflows/Continuous%20integration/badge.svg)
![Rust](https://github.com/jdambron/kindlenotes2anki/workflows/Rust/badge.svg)

Tool to convert kindle clippings file to CSV for importing in [Anki](https://apps.ankiweb.net/)

## Disclaimer

This is mostly a project to play around with [Rust](https://www.rust-lang.org/) and learn it on something real.
So there are probably lots of things that can be improved.

## Build the project

```Shell
cargo build --release
```

## Run

```Shell
kindlenotes2anki My\ Clippings.txt > file.csv
```
