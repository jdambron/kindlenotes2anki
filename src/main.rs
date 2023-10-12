mod app_config;
mod connect;
mod csv_writer;
mod my_clippings_parser;
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The path to the clippings txt file to read
    clippings: PathBuf,
    /// Use AnkiConnect, if not provided will generate a CSV output
    #[arg(short, long)]
    connect: bool,
    /// The path to a config file, if not provided will use defaults
    #[arg(long)]
    config: Option<PathBuf>,
}

/// Representation of a note
pub struct Note {
    /// Title of the book
    title: String,
    /// Tidied content of the note
    tidied_note: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    app_config::AppConfig::init(args.config)?;
    let notes = my_clippings_parser::parse_clippings(args.clippings)
        .context("Failed to parse clippings")?;
    if args.connect {
        connect::write_notes_ankiconnect(notes).context("Failed to send notes to AnkiConnect")?;
    } else {
        csv_writer::write_csv(notes).context("Failed to write notes to CSV")?;
    }
    Ok(())
}
