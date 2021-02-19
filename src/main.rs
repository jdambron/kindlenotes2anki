mod app_config;
mod connect;
mod csv_writer;
mod my_clippings_parser;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(
    name = "kindlenotes2anki",
    about = "A tool to import kindle clippings file to Anki"
)]
struct Cli {
    /// The path to the clippings txt file to read
    #[structopt(parse(from_os_str))]
    clippings: PathBuf,
    /// Use AnkiConnect, if not provided will generate a CSV output
    #[structopt(short, long)]
    connect: bool,
    /// The path to a config file, if not provided will use defaults
    #[structopt(parse(from_os_str), long)]
    config: Option<PathBuf>,
}

/// Representation of a note
pub struct Note {
    /// Title of the book
    title: String,
    /// Tidied content of the note
    tidied_note: String,
}

fn main() {
    let config_contents = include_str!("resources/default_config.toml");
    app_config::AppConfig::init(Some(config_contents)).unwrap();
    let args = Cli::from_args();
    let notes = my_clippings_parser::parse_clippings(args.clippings);
    let _c = app_config::AppConfig::merge_config(args.config);
    if args.connect {
        connect::write_notes_ankiconnect(notes);
    } else {
        csv_writer::write_csv(notes);
    }
}
