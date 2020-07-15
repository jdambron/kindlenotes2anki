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

fn parse_note(note: impl ToString) -> Option<(String, String)> {
    let lines: Vec<String> = note.to_string().lines().map(|x| x.to_string()).collect();
    let title: String = lines
        .iter()
        .take(1)
        .map(|x| x.trim().trim_start_matches("\u{feff}"))
        .collect();
    let tidied_note: String = lines.iter().skip(1).map(tidy_note_line).collect();
    if title.is_empty() || tidied_note.is_empty() {
        None
    } else {
        Some((title, tidied_note))
    }
}

fn tidy_note_line(line: impl ToString) -> String {
    let linestr = line.to_string();
    if linestr.starts_with("- Votre surlignement")
        || linestr.starts_with("- Votre signet")
        || linestr.starts_with("- Votre note")
        || linestr.is_empty()
    {
        "".to_string()
    } else {
        linestr + "\n"
    }
}
