#![feature(let_chains)]

use std::io::{BufWriter, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::{fs, io};

use chrono::NaiveDateTime;
use clap::Parser;

use log::Entry;
use parse::logs_in_dir;

mod log;
mod parse;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Args {
    /// Parses a combat.log file
    File(FileArgs),
    /// Parses all logs in the sub directories. Path can be inferred
    Directory(DirectoryArgs),
}

#[derive(Parser, Debug)]
struct FileArgs {
    /// The input combat.log file
    #[clap()]
    input: PathBuf,
    /// The date of the combat.log file
    #[clap(short, long)]
    date: NaiveDateTime,
    /// The output object file
    #[clap(short, long)]
    output: PathBuf,
}

#[derive(Parser, Debug)]
struct DirectoryArgs {
    /// The directory containing the logs. Default '${Documents}/My Games/Crossout/logs'
    #[clap(default_value = "")]
    input: PathBuf,
    /// The output directory for object files
    #[clap(short, long)]
    output: PathBuf,
}

fn main() {
    if let Err(e) = match Args::parse() {
        Args::File(p) => parse_log(p),
        Args::Directory(d) => parse_logs_in_dir(d),
    } {
        println!("{}", e);
    }
}

fn parse_log(args: FileArgs) -> Result<(), Error> {
    if !args.input.is_file() {
        return Err(Error::FileNotFound(args.input));
    }
    if args.output.parent().map(|p| p.is_dir()).unwrap_or(false) {
        return Err(Error::DirNotFound(args.output));
    }
    let (messages, errors) = parse::parse_logs(vec![(args.input, args.date.date(), 0..usize::MAX)]);
    write_output(&args.output, messages, errors)?;
    Ok(())
}

fn parse_logs_in_dir(args: DirectoryArgs) -> Result<(), Error> {
    let input = amortized_logs_dir(args.input)?;
    if !input.is_dir() {
        return Err(Error::LogDirNotInferred);
    }
    if !args.output.is_dir() {
        fs::create_dir_all(&args.output)?;
    }
    let logs = logs_in_dir(input)?;
    let (messages, errors) = parse::parse_logs(logs.into_iter().map(|(p, dt)| (p, dt.date(),
     0..usize::MAX)));

    let mut output = args.output;
    output.push("combat.log.bin");
    write_output(&output, messages, errors)
}

fn amortized_logs_dir(dir: PathBuf) -> Result<PathBuf, Error> {
    if dir.as_os_str().is_empty() {
        let mut dir = dirs::document_dir().ok_or(Error::LogDirNotInferred)?;
        dir.push("My Games");
        dir.push("Crossout");
        dir.push("logs");
        Ok(dir)
    } else {
        Ok(dir)
    }
}

fn write_output(output: &Path, messages: Vec<Entry>, errors: Vec<String>) -> Result<(), Error> {
    let writer = fs::File::create(output)?;
    bincode::serialize_into(BufWriter::new(writer), &messages)?;

    if !errors.is_empty() {
        let writer = fs::File::create(output.with_extension("errors.log"))?;
        let mut writer = BufWriter::new(writer);
        for err in errors {
            writer.write_all(format!("{}\n", err).as_bytes())?;
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum Error {
    LogDirNotInferred,
    FileNotFound(PathBuf),
    DirNotFound(PathBuf),
    File(io::Error),
    Ser(bincode::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::LogDirNotInferred => write!(f, "Log directory could not be inferred"),
            Error::FileNotFound(p) => write!(f, "File `{}` not found", p.display()),
            Error::DirNotFound(p) => write!(f, "Directory `{}` not found", p.display()),
            Error::File(e) => write!(f, "{}", e),
            _ => write!(f, "Unexpected error occurred"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::File(e)
    }
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Error::Ser(e)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_directory_logs() {
        parse_logs_in_dir(DirectoryArgs { input: "".into(), output: "./publish".into() }).expect("nope");
    }
}
