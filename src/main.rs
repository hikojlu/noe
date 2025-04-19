use clap::{Parser, Subcommand};
use home::home_dir;
use noe::*;
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

    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List notes
    List {
        #[arg(short, long)]
        all: bool,
    },
    /// Create a note
    New {
        note: String,
        #[arg(short, long)]
        number: Option<u16>,
    },
    /// Mark note done
    Done { number: u16 },
    /// Remove note
    Remove { number: u16 },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (file, command, verbose) = {
        let args = Args::parse();
        let file_buf = args.file.unwrap_or_else(|| home_dir().expect("homeless"));
        let file_buf = if file_buf.is_dir() {
            file_buf.join(DEFAULT_FILENAME)
        } else {
            file_buf
        };
        (file_buf, args.command.unwrap(), args.verbose)
    };

    let mut handle = Handle::open(file)?;

    match command {
        Command::List { all } => {
            let out = handle
                .list_notes()?
                .as_vec()
                .iter()
                .filter_map(|Note { number, text, done }| {
                    if !done || all {
                        Some((number, text))
                    } else {
                        None
                    }
                })
                .map(|(n, txt)| {
                    let note = txt.replace("\\n", "\n");
                    format!("  #{n}:\n") + &note + &"\n"
                })
                .collect::<String>();
            print!("{}", out);
        }
        Command::New { note, number } => {
            let number = handle.new_note(note, number)?;
            if verbose {
                println!("Added note #{number}")
            }
        }
        Command::Done { number } => {
            handle.done_note(number)?;
            if verbose {
                println!("Marked note #{number} done")
            }
        }
        Command::Remove { number } => {
            handle.remove_note(number)?;
            if verbose {
                println!("Removed note #{number}")
            }
        }
    }
    Ok(())
}
