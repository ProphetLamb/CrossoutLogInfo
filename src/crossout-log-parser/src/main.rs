#![feature(let_chains)]

use chrono::{NaiveDate, NaiveDateTime};
use clap::Parser;
use log::{parse_entry, Entry};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::{
    fs,
    io::{self, BufRead, BufReader},
};

#[macro_use]
extern crate tokio;

mod log;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Args {
    Parse(ParseArgs),
    Directory(DirectoryArgs),
}

#[derive(Parser, Debug)]
struct ParseArgs {
    #[clap(short, long)]
    input: PathBuf,
    #[clap(short, long)]
    date: NaiveDate,
    #[clap(short, long)]
    output: PathBuf,
}

#[derive(Parser, Debug)]
struct DirectoryArgs {
    #[clap(default_value = "")]
    directory: PathBuf,
}

fn main() {
    if let Err(e) = match Args::parse() {
        Args::Parse(p) => parse_log(p),
        Args::Directory(d) => directory_logs(d),
    } {
        println!("{}", e);
    }
}

fn directory_logs(args: DirectoryArgs) -> Result<(), Error> {
    let mut log_dirs = Vec::default();
    for dir in args
        .directory
        .read_dir()?
        .flatten()
        .filter(|sub| sub.file_type().map_or(false, |t| t.is_dir()))
    {
        let name = dir.file_name();
        if let Some(name) = name.to_str() && let Ok(date) = NaiveDateTime::parse_from_str(name, "%Y.%m.%d %H.%M.%S") {
            log_dirs.push((dir, date))
        }
    }

    for (dir, datetime) in log_dirs {
        let mut combat_log = dir.path();
        combat_log.push("combat.log");
        let output = format!("./publish/{}.json", datetime.format("%Y.%m.%d %H.%M.%S"));
        let s = combat_log.to_str();
        parse_log(ParseArgs {
            input: combat_log,
            date: datetime.date(),
            output: output.into(),
        })?;
    }

    Ok(())
}

fn parse_log(args: ParseArgs) -> Result<(), Error> {
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
            .write_all(format!("{:?}\n", msg).as_bytes())
            .expect("unable to write to output file");
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
    None,
    File(io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
    use std::{env, fmt::Debug, path, str::FromStr};

    use chrono::NaiveDate;

    use crate::log::DamageFlag;

    use super::*;

    #[test]
    fn test_parse_log() {
        let input = PathBuf::from_str("./scripts/unique_combat.log").expect("nope");
        let output = PathBuf::from_str("./output.log").expect("nope");
        let date = NaiveDate::from_ymd(2000, 1, 1);
        parse_log(ParseArgs {
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
        directory_logs(DirectoryArgs { directory }).expect("nope");
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
