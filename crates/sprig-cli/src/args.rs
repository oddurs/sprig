//! The command line, and the one place the program reads the clock.
//!
//! Every answer Sprig gives depends on the date only through `--today`, so any
//! output can be reproduced by passing the day it was produced on. The default
//! comes from the system clock here and nowhere else; `scripts/task lint`
//! refuses a clock read in any other file.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use sprig_core::Date;

const AFTER_HELP: &str = "\
Paths are .sprig files or directories; a directory means every .sprig file
under it. With no path, the current directory.

Exit codes:
  0  fine
  1  findings: `check` found errors (or warnings, with --strict), or `next`
     found nothing ready
  2  usage error, or a path that could not be read";

#[derive(Debug, Parser)]
#[command(
    name = "sprig",
    version,
    about = "Plain-text plans for people and coding agents",
    after_help = AFTER_HELP,
    arg_required_else_help = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// The date to answer for, as YYYY-MM-DD [default: the system date]
    #[arg(long, global = true, value_name = "DATE", env = "SPRIG_TODAY", value_parser = parse_date)]
    pub today: Option<Date>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// What can be done now: open, unblocked work, most urgent first
    #[command(after_help = AFTER_HELP)]
    Next(NextArgs),
    /// Report broken grafts, loops and unknown references, for editors and CI
    #[command(after_help = AFTER_HELP)]
    Check(CheckArgs),
}

#[derive(Debug, Args)]
pub struct NextArgs {
    /// Plans to read
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Only work whose owner is NAME (with or without the @)
    #[arg(long, value_name = "NAME")]
    pub who: Option<String>,

    /// Show at most N items
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    /// Plans to read
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Exit 1 on warnings too
    #[arg(long)]
    pub strict: bool,
}

fn parse_date(s: &str) -> Result<Date, String> {
    Date::parse_iso(s).ok_or_else(|| format!("{s} is not a date; write it as YYYY-MM-DD"))
}

/// `--today`, or the system's local date when it is not given.
pub fn today(cli: &Cli) -> Date {
    cli.today.unwrap_or_else(|| {
        let d = jiff::Zoned::now().date();
        Date::new(i32::from(d.year()), d.month() as u8, d.day() as u8)
            .expect("the system clock gives a real date")
    })
}
