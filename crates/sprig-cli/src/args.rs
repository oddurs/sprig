//! The command line, and the one place the program reads the clock.
//!
//! Every answer Sprig gives depends on the date only through `--today`, so any
//! output can be reproduced by passing the day it was produced on. The default
//! comes from the system clock here and nowhere else; `scripts/task lint`
//! refuses a clock read in any other file.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use sprig_core::Date;

const AFTER_HELP: &str = "\
Paths are .sprig files or directories; a directory means every .sprig file
under it. With no path, the current directory.

Exit codes:
  0  fine
  1  findings: `check` found errors (or warnings, with --strict), `next`
     found nothing ready, `find` matched nothing, or `fmt --check` found
     files to change
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
    #[arg(long, global = true, value_name = "DATE", env = "SPRIG_TODAY", hide_env_values = true, value_parser = parse_date)]
    pub today: Option<Date>,

    /// When to colour the marks [default: auto, which is off when NO_COLOR is set]
    #[arg(long, global = true, value_name = "WHEN", value_enum, default_value_t = ColorChoice::Auto, hide_default_value = true)]
    pub color: ColorChoice,

    /// Tab-separated output for scripts: a ref first, no colour, nothing cut
    #[arg(long, global = true)]
    pub plain: bool,

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
    /// Normalise indentation, write out typed dates, trim trailing space
    #[command(after_help = AFTER_HELP)]
    Fmt(FmtArgs),
    /// Mark items done, or reopen them, or set any mark; a recurring item moves its due date
    #[command(after_help = TICK_HELP)]
    Tick(TickArgs),
    /// A plan as a tree, with progress, dates and what each item waits on; or
    /// the whole folder, a line per file
    #[command(after_help = AFTER_HELP)]
    Tree(TreeArgs),
    /// Why an item is blocked, all the way down, with the file and line of each link
    #[command(after_help = WHY_HELP)]
    Why(WhyArgs),
    /// Search items by text, @who, #tag, is:STATE and due:DATE
    #[command(after_help = FIND_HELP)]
    Find(FindArgs),
    /// The plans as data: the tree as JSON, dated work as iCalendar, items as
    /// CSV, the dependency graph for Graphviz or Mermaid
    #[command(after_help = AFTER_HELP)]
    Export(ExportArgs),
    /// Convert a Markdown checklist, a todo.txt file or GitHub issues to Sprig
    #[command(after_help = IMPORT_HELP)]
    Import(ImportArgs),
    /// Run the language server on stdin and stdout, for editors
    Lsp,
    /// Serve the plans to a coding agent over the Model Context Protocol, on stdin and stdout
    #[command(after_help = MCP_HELP)]
    Mcp(McpArgs),
    /// Work the plan from one screen: the tree, next up and the agenda, with vim keys
    #[command(after_help = TUI_HELP)]
    Tui(TuiArgs),
    /// Print a shell completion script, or the manual page in roff
    Generate(GenerateArgs),
    /// Candidates for shell completion: refs and people in the current folder
    #[command(name = "__complete", hide = true)]
    Complete(CompleteArgs),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

const WHY_HELP: &str = "\
Exit codes:
  0  the item is not blocked
  1  the item is blocked; the chain says by what
  2  usage error, or a ref that names no item or several";

#[derive(Debug, Args)]
pub struct TreeArgs {
    /// Plans to show
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Show N levels at most
    #[arg(long, value_name = "N")]
    pub depth: Option<usize>,

    /// Leave out done and dropped items
    #[arg(long)]
    pub hide_done: bool,

    /// The whole folder on one screen: a line per file with its progress, and
    /// what is waiting and late
    #[arg(long, conflicts_with_all = ["depth", "hide_done"])]
    pub summary: bool,

    /// With --summary: one short line for a shell prompt or a status bar; exit
    /// 1 if anything is late
    #[arg(long, requires = "summary")]
    pub oneline: bool,

    /// With --summary: a row per plan as JSON: file, title, done, total, ready,
    /// waiting, late and errors
    #[arg(long, requires = "summary", conflicts_with = "oneline")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct WhyArgs {
    /// The item to explain
    #[arg(value_name = "REF")]
    pub item: String,

    /// The plans the ref is in
    #[arg(long = "in", value_name = "PATH", default_value = ".")]
    pub plans: PathBuf,
}

/// What `sprig generate` prints.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Generated {
    /// A completion script for bash
    Bash,
    /// A completion script for zsh
    Zsh,
    /// A completion script for fish
    Fish,
    /// A completion script for elvish
    Elvish,
    /// A completion script for PowerShell
    Powershell,
    /// The manual page, in roff
    Man,
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    /// A shell to complete for, or man for the manual page
    #[arg(value_enum, value_name = "WHAT")]
    pub what: Generated,
}

#[derive(Debug, Args)]
pub struct CompleteArgs {
    /// What to list: refs or people
    pub what: String,
}

const TICK_HELP: &str = "\
Ticking an item done marks it x, and ticking a done item reopens it. --mark
sets any status mark instead: - ~ x / > or ?. Everything else is a change to
the file, made in an editor: the file is the truth.

A ref names one item, from exact to forgiving:
  ^signed          an anchor, in any file
  lease^signed     an anchor in one file
  lease.sprig:13   the item on a line, as compilers print it
  \"grease trap\"    words from the item's text, matching one item
  -                refs from standard input, one per line

Every ref is resolved before anything is written: if one is unknown or
ambiguous, nothing changes, the candidates are listed, and the exit code is 2.

Exit codes:
  0  ticked
  2  usage error, a ref that names no item or several, an item that cannot
     take the mark, or a path that could not be read or written";

const FIND_HELP: &str = "\
A query is words and filters, all of which an item must match:
  oven               words from the item's text, ignoring case
  @teo  #kitchen     its people (inherited ones included) and its tags
  is:open            open, todo, doing, done, dropped, later, ask or blocked
  due:<fri           its own due date: <, <=, >, >= or exactly, as a date
                     or a shortcut such as fri, +2w or oct3

Exit codes:
  0  something matched
  1  nothing matched
  2  usage error, an unknown filter, or a path that could not be read";

const IMPORT_HELP: &str = "\
The format comes from the file's extension unless --from names it: .md is a
Markdown checklist, .txt is todo.txt, and .json is the output of
  gh issue list --json number,title,state,labels,assignees,milestone
The Sprig file is printed; redirect it where it belongs. Nothing is read from
the network.

Exit codes:
  0  converted
  2  usage error, or a file that could not be read or understood";

const MCP_HELP: &str = "\
Register it with an agent's MCP client; `sprig mcp --config` prints the
snippet, and `sprig mcp --instructions --write AGENTS.md` tells the agent how
the repository plans. Tools: list_plans, show_tree, next_items, find_items, check_plans,
tick_items, mark_item, add_items, answer_question and set_field.

Every write goes through the same edit API as `sprig tick`, is refused when it
would add an error `sprig check` reports, and returns the diff it applied.";

const TUI_HELP: &str = "\
Keys: j k move, h l fold, x tick, ~ doing, a A add, e edit, u undo,
1 2 3 tree, next up and agenda, / filter, ? help, q quit.

Every write goes through the same edit API as `sprig tick`, and is refused if
the line changed on disk since the TUI read it. The files are read again
whenever one changes.";

#[derive(Debug, Args)]
pub struct TuiArgs {
    /// A plan or a folder of plans [default: .]
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Print one frame, WIDTHxHEIGHT such as 100x30, as text and exit
    #[arg(long, value_name = "WxH")]
    pub screenshot: Option<String>,

    /// With --screenshot: keys to press first, such as "jjx" or "/oven<Enter>"
    #[arg(long, value_name = "KEYS", requires = "screenshot")]
    pub keys: Option<String>,
}

#[derive(Debug, Args)]
pub struct McpArgs {
    /// The folder of plans to serve
    #[arg(long = "in", value_name = "PATH", default_value = ".")]
    pub plans: PathBuf,

    /// The most items add_items may add, in one call and in the whole session
    #[arg(long, value_name = "N", default_value_t = 20)]
    pub max_new_items: usize,

    /// Print the configuration that registers this server with an MCP client, and exit
    #[arg(long, conflicts_with = "instructions")]
    pub config: bool,

    /// Print the instructions that tell a coding agent how this repository
    /// plans, and exit
    #[arg(long)]
    pub instructions: bool,

    /// With --instructions: keep them in FILE, such as AGENTS.md or CLAUDE.md,
    /// between markers; a second run replaces the block, and changes nothing if
    /// it is current
    #[arg(long, value_name = "FILE", requires = "instructions")]
    pub write: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct FindArgs {
    /// Words and filters
    #[arg(value_name = "QUERY", required = true)]
    pub query: Vec<String>,

    /// The plans to search
    #[arg(long = "in", value_name = "PATH", default_value = ".")]
    pub plans: PathBuf,

    /// Matches as JSON: a ref, a path and a node of the tree format each
    #[arg(long)]
    pub json: bool,
}

/// The marks `sprig tick --mark` sets, as characters or words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarkValue(pub sprig_core::Mark);

pub(crate) fn parse_mark(s: &str) -> Result<MarkValue, String> {
    use sprig_core::Mark;
    Ok(MarkValue(match s {
        "-" | "todo" => Mark::Todo,
        "~" | "doing" => Mark::Doing,
        "x" | "done" => Mark::Done,
        "/" | "dropped" => Mark::Dropped,
        ">" | "later" => Mark::Later,
        "?" | "ask" | "question" => Mark::Question,
        _ => {
            return Err(format!(
                "{s} is not a status mark; use - ~ x / > or ? (or todo, doing, done, dropped, later, ask)"
            ));
        }
    }))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ExportFormat {
    /// Each plan's tree, in the tree format (spec Appendix D)
    Json,
    /// An all-day event per dated item, for a calendar to subscribe to
    Ics,
    /// Every item, one row each
    Csv,
    /// The dependency graph, for Graphviz
    Dot,
    /// The dependency graph, for Mermaid
    Mermaid,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    /// Plans to read
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// What to write
    #[arg(long, value_enum)]
    pub format: ExportFormat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ImportFormat {
    Markdown,
    Todotxt,
    Github,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    /// The file to convert, or - for standard input
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// The file's format [default: from its extension]
    #[arg(long, value_enum)]
    pub from: Option<ImportFormat>,

    /// The title for a format that has none [default: from the file name]
    #[arg(long, value_name = "TEXT")]
    pub title: Option<String>,
}

#[derive(Debug, Args)]
pub struct FmtArgs {
    /// Plans to format
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Write nothing; list the files that would change, and exit 1 if any
    #[arg(long)]
    pub check: bool,
}

#[derive(Debug, Args)]
pub struct TickArgs {
    /// Items to tick
    #[arg(value_name = "REF", required = true)]
    pub refs: Vec<String>,

    /// The plans the refs are in
    #[arg(long = "in", value_name = "PATH", default_value = ".")]
    pub plans: PathBuf,

    /// Set this mark instead: - ~ x / > or ?, or todo, doing, done, dropped,
    /// later, ask
    #[arg(long, value_name = "MARK", value_parser = parse_mark, allow_hyphen_values = true)]
    pub mark: Option<MarkValue>,
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

    /// Ready work as JSON: a ref, a path and a node of the tree format each
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    /// Plans to read
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Exit 1 on warnings too
    #[arg(long)]
    pub strict: bool,

    /// Diagnostics as JSON, in the tree format's shape; the exit code is the same
    #[arg(long)]
    pub json: bool,
}

fn parse_date(s: &str) -> Result<Date, String> {
    Date::parse_iso(s).ok_or_else(|| format!("{s} is not a date; write it as YYYY-MM-DD"))
}

/// `--today`, or the system's local date when it is not given.
pub fn today(cli: &Cli) -> Date {
    cli.today.unwrap_or_else(system_date)
}

/// Today, asked again each time: a language server runs past midnight, and
/// a start date that was ahead yesterday is not today. `--today` still fixes
/// it, so an editor session can be reproduced.
pub fn clock(cli: &Cli) -> impl Fn() -> Date + use<> {
    let fixed = cli.today;
    move || fixed.unwrap_or_else(system_date)
}

fn system_date() -> Date {
    let d = jiff::Zoned::now().date();
    Date::new(i32::from(d.year()), d.month() as u8, d.day() as u8)
        .expect("the system clock gives a real date")
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::Cli;

    /// The reference on the site is the binary's own help, so a command or an
    /// option without help would be a hole in it.
    #[test]
    fn every_command_and_option_has_help() {
        let cli = Cli::command();
        let mut missing = Vec::new();
        for cmd in cli.get_subcommands().filter(|c| !c.is_hide_set()) {
            if cmd
                .get_about()
                .is_none_or(|a| a.to_string().trim().is_empty())
            {
                missing.push(format!("sprig {}", cmd.get_name()));
            }
            for arg in cmd.get_arguments().filter(|a| !a.is_hide_set()) {
                if arg.get_id() == "help" || arg.get_id() == "version" {
                    continue;
                }
                if arg
                    .get_help()
                    .is_none_or(|h| h.to_string().trim().is_empty())
                {
                    missing.push(format!("sprig {} {}", cmd.get_name(), arg.get_id()));
                }
            }
        }
        assert!(
            missing.is_empty(),
            "no help for:\n  {}",
            missing.join("\n  ")
        );
    }
}
