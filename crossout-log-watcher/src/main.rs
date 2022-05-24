#![feature(let_chains)]

use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::{
    fs,
    io::{self, BufRead, BufReader},
};

use chrono::{NaiveDate, NaiveDateTime};
use clap::Parser;

use log::Entry;
use parse::parse_entry;
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
    date: NaiveDate,
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

fn parse_logs_in_dir(args: DirectoryArgs) -> Result<(), Error> {
    if !dir_exists(&args.input) {
        return Err(Error::DocDirNotFound);
    }
    if !dir_exists(&args.output) {
        fs::create_dir_all(&args.output)?;
    }
    let logs = logs_in_dir(args.input)?;
    parse_log_files(logs, &args.output)
}

fn dir_exists(path: &Path) -> bool {
    if let Ok(meta) = path.metadata() {
        meta.is_dir()
    } else {
        false
    }
}

fn logs_in_dir(input: PathBuf) -> Result<Vec<(fs::DirEntry, NaiveDateTime)>, Error> {
    let mut log_dirs = Vec::default();
    for dir in amortized_logs_dir(input)?
        .read_dir()?
        .flatten()
        .filter(|sub| sub.file_type().map_or(false, |t| t.is_dir()))
    {
        let name = dir.file_name();
        if let Some(name) = name.to_str() && let Ok(date) = NaiveDateTime::parse_from_str(name, "%Y.%m.%d %H.%M.%S") {
            log_dirs.push((dir, date))
        }
    }

    Ok(log_dirs)
}

fn parse_log_files(log_dirs: Vec<(fs::DirEntry, NaiveDateTime)>, output_dir: &Path) -> Result<(), Error> {
    for (dir, datetime) in log_dirs {
        let mut input = dir.path();
        input.push("combat.log");
        let mut output = output_dir.to_path_buf();
        output.push(format!("{}.bin", datetime.format("%Y.%m.%d %H.%M.%S")));
        parse_log(FileArgs {
            input,
            date: datetime.date(),
            output,
        })?;
    };
    Ok(())
}

fn amortized_logs_dir(dir: PathBuf) -> Result<PathBuf, Error> {
    if dir.as_os_str().is_empty() {
        let mut dir = dirs::document_dir().ok_or(Error::DocDirNotFound)?;
        dir.push("My Games");
        dir.push("Crossout");
        dir.push("logs");
        Ok(dir)
    } else {
        Ok(dir)
    }
}

fn parse_log(args: FileArgs) -> Result<(), Error> {
    let input = open_file(&args.input)?;
    let mut messages = Vec::with_capacity(1024);
    let mut errors = Vec::with_capacity(1024);
    parse_messages(input, args.date, &mut messages, &mut errors);
    write_output(&args.output, messages, errors)?;
    Ok(())
}

fn open_file(input: &Path) -> Result<BufReader<fs::File>, io::Error> {
    let file = fs::File::open(input)?;
    let reader = io::BufReader::new(file);
    Ok(reader)
}

fn parse_messages<R>(
    source: BufReader<R>,
    date: NaiveDate,
    messages: &mut Vec<Entry>,
    errors: &mut Vec<String>,
) where
    R: std::io::Read,
{
    for line in source.lines().flatten() {
        if let Ok((_, message)) = parse_entry::<()>(date)(&line) {
            messages.push(message);
        } else if !line.is_empty() {
            errors.push(line.to_owned());
        }
    }
}

fn write_output(output: &Path, messages: Vec<Entry>, errors: Vec<String>) -> Result<(), Error> {
    let writer = fs::File::create(output)?;
    let mut writer = BufWriter::new(writer);
    for msg in messages {
        writer
            .write_all(format!("{:?}\n", msg).as_bytes())?;
    }
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
    DocDirNotFound,
    File(io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DocDirNotFound => write!(f, "Document directory not found"),
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

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use chrono::NaiveDate;

    use crate::log::DamageFlag;

    use super::*;

    #[test]
    fn test_parse_log() {
        let input = PathBuf::from_str("./scripts/unique_combat.log").expect("nope");
        let output = PathBuf::from_str("./output.log").expect("nope");
        let date = NaiveDate::from_ymd(2000, 1, 1);
        parse_log(FileArgs {
            input,
            date,
            output,
        })
        .ok();
    }

    #[test]
    fn test_directory_logs() {
        let mut directory = dirs::document_dir().expect("nope");
        directory.push("My Games");
        directory.push("Crossout");
        directory.push("logs");
        parse_logs_in_dir(DirectoryArgs { input: directory, output: "./publish".into() }).expect("nope");
    }

    #[test]
    fn test_damage_flags() {
        let flags = vec![
            "DMG_DIRECT",
            "HUD_IMPORTANT",
            "HUD_HIDDEN",
            "DMG_GENERIC",
            "SUICIDE",
            "SUICIDE_DESPAWN",
            "DMG_BLAST",
            "CONTINUOUS",
            "DMG_ENERGY",
            "CONTACT",
            "DMG_COLLISION",
            "DMG_FLAME",
        ];
        for flag in flags {
            let result = str::parse::<DamageFlag>(flag);
            if let Err(e) = result {
                println!("{:?} failed with {:?}", flag, e)
            }
        }
    }
}
