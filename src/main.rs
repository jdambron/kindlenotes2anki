mod app_config;
mod connect;
mod csv_writer;
mod my_clippings_parser;
mod note;
use anyhow::{Context, Result};
use app_config::AppConfig;
use clap::Parser;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the clippings txt file to read
    clippings: PathBuf,
    /// Use `AnkiConnect`, if not provided will generate a CSV output
    #[arg(short, long)]
    use_anki_connect: bool,
    /// Write CSV to this file instead of stdout (ignored with `--use-anki-connect`)
    #[arg(short = 'o', long, value_name = "PATH")]
    output: Option<PathBuf>,
    /// The path to a config file, if not provided will use defaults
    #[arg(long)]
    config: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let config = AppConfig::new(args.config).context("Failed to initialize app config")?;
    let notes = my_clippings_parser::parse_clippings(&args.clippings, &config)
        .context("Failed to parse clippings")?;
    if args.use_anki_connect {
        let count = connect::add_notes(&notes, &config.anki)
            .context("Failed to send notes to AnkiConnect")?;
        writeln!(io::stderr(), "Imported {count} notes")?;
    } else {
        let count = match args.output {
            Some(path) => {
                let file = File::create(&path)
                    .with_context(|| format!("Failed to create output file: {}", path.display()))?;
                csv_writer::write_csv(&notes, file).context("Failed to write notes to CSV")?
            }
            None => csv_writer::write_csv(&notes, io::stdout())
                .context("Failed to write notes to CSV")?,
        };
        writeln!(io::stderr(), "Exported {count} notes")?;
    }
    Ok(())
}
