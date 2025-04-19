use clap::{Parser, Subcommand};
use home::home_dir;
use noe::*;
use std::fs;
use std::path::PathBuf;

const DEFAULT_FILENAME: &'static str = ".notes.db";

#[derive(Debug, Parser)]
#[command(
    name = "noe",
    version = "0.1.0",
    about = "Simple note-taking cli app",
    long_about = None,

    arg_required_else_help = true,
    subcommand_required = true,
    infer_subcommands = true,
)]
struct Args {
    #[arg(short, long)]
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List notes
    List {
        /// Pist all notes
        #[arg(short, long)]
        all: bool,
        /// Pist only done notes
        #[arg(short, long)]
        done: bool,
    },
    /// Create a note
    New {
        note: String,
        #[arg(short, long)]
        number: Option<u16>,
    },
    /// Mark note done
    Done { number: u16 },
    /// Mark note undone
    Undone { number: u16 },
    /// Remove note
    Remove { number: u16 },
    /// Clean all notes
    Explode,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (file, command) = {
        let args = Args::parse();
        let file_buf = args.file.unwrap_or_else(|| home_dir().expect("homeless"));
        let file_buf = if file_buf.is_dir() {
            file_buf.join(DEFAULT_FILENAME)
        } else {
            file_buf
        };
        (file_buf, args.command)
    };

    let mut handle = Handle::open(&file)?;

    match command {
        Command::List {
            all,
            done: only_done,
        } => {
            let out = handle
                .list_notes()?
                .as_vec()
                .iter()
                .filter_map(|Note { number, text, done }| match (all, done, only_done) {
                    (true, ..) => Some((number, text, done)),
                    (.., true, true) => Some((number, text, done)),
                    (.., false, false) => Some((number, text, done)),
                    _ => None,
                })
                .map(|(n, txt, done)| {
                    let done = if *done { " done!" } else { "" };
                    let note = txt.replace("\\n", "\n");
                    format!("  #{n}:{done}\n") + &note + &"\n"
                })
                .collect::<String>();
            print!("{}", out);
        }
        Command::New { note, number } => {
            let number = handle.new_note(note, number)?;
            println!("Added note #{number}")
        }
        Command::Done { number } => {
            handle.done_note(number)?;
            println!("Marked note #{number} done")
        }
        Command::Undone { number } => {
            handle.undone_note(number)?;
            println!("Marked note #{number} undone")
        }
        Command::Remove { number } => {
            handle.remove_note(number)?;
            println!("Removed note #{number}")
        }
        Command::Explode => {
            fs::remove_file(&file)?;
            println!("Notes exploded")
        }
    }
    Ok(())
}
