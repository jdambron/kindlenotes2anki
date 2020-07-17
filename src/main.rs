use std::io;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    /// The path to the file to read
    #[structopt(parse(from_os_str))]
    clippings: PathBuf,
}

fn main() {
    let args = Cli::from_args();
    parse_clippings(args.clippings);
}

fn parse_clippings(filename: PathBuf) {
    let mut wtr = csv::Writer::from_writer(io::stdout());
    let separator = "==========\r\n";
    let content = std::fs::read_to_string(filename).unwrap();
    let notes = content.split(separator);
    for note in notes {
        if let Some((title, tidied_note)) = parse_note(note) {
            wtr.write_record(&[title, tidied_note]).unwrap();
        }
    }
    wtr.flush().unwrap();
}

fn parse_note(note: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = note.lines().collect();
    let title: String = lines
        .iter()
        .take(1)
        .map(|x| x.trim().trim_start_matches("\u{feff}"))
        .collect();
    let tidied_note: String = lines
        .iter()
        .skip(1)
        .filter(|l| !is_useless_line(l))
        .map(|l| l.to_string())
        .collect::<Vec<String>>()
        .join("\n");
    if title.is_empty() || tidied_note.is_empty() {
        None
    } else {
        Some((title, tidied_note))
    }
}

fn is_useless_line(line: &str) -> bool {
    line.starts_with("- Votre surlignement")
        || line.starts_with("- Votre signet")
        || line.starts_with("- Votre note")
        || line.is_empty()
}
