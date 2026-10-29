use std::io::{self, Write};
use std::process::ExitCode;

use clap::{CommandFactory, Parser};

mod agent;
mod args;
mod data;
mod load;
mod mcp;
mod out;
mod tui;
mod views;
mod write;

use args::{Cli, Command, Generated};
use out::Out;

fn main() -> ExitCode {
    // Plans nest as deep as their author likes (§3.2) and the core recurses
    // over them, so it runs with room to spare rather than the main thread's
    // few megabytes. Depth needs indentation, which needs bytes, so this is
    // generous for any file that fits in memory.
    let run = std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(run)
        .expect("a thread can be started");
    run.join().unwrap_or(ExitCode::from(2))
}

/// Commands removed before 1.0 (cairn 0154), and what does their work now.
/// The table goes at 1.0, when the command line stops changing.
const REMOVED: &[(&str, &str)] = &[
    ("status", "sprig tree --summary"),
    ("agenda", "sprig find is:open 'due:<=+2w'"),
    ("mark", "sprig tick REF --mark MARK"),
    ("answer", "an `= ` line under the question, in your editor"),
    ("add", "a new line in the file, in your editor"),
    ("edit", "your editor, or `e` in sprig tui"),
    ("graph", "sprig export --format dot (or mermaid)"),
    ("agent", "sprig mcp --instructions"),
    ("completions", "sprig generate SHELL"),
    ("man", "sprig generate man"),
];

fn run() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            if e.kind() == clap::error::ErrorKind::InvalidSubcommand
                && let Some(word) = std::env::args().skip(1).find(|a| !a.starts_with('-'))
                && let Some((_, now)) = REMOVED.iter().find(|(old, _)| *old == word)
            {
                eprintln!("sprig: `sprig {word}` is gone; use {now}");
                return ExitCode::from(2);
            }
            e.exit()
        }
    };
    let today = args::today(&cli);
    let out = Out::new(cli.color, cli.plain);
    let result = match &cli.command {
        Command::Next(a) => views::next(a, today, out),
        Command::Check(a) => views::check(a, today, out),
        Command::Tree(a) => views::tree(a, today, out),
        Command::Why(a) => views::why(a, today, out),
        Command::Find(a) => data::find(a, today, out),
        Command::Export(a) => data::export(a, today),
        Command::Import(a) => data::import(a),
        Command::Fmt(a) => write::fmt(a, today),
        Command::Tick(a) => write::tick(a, today),
        Command::Generate(a) => match a.what {
            Generated::Man => man(),
            Generated::Bash => completions(clap_complete::Shell::Bash),
            Generated::Zsh => completions(clap_complete::Shell::Zsh),
            Generated::Fish => completions(clap_complete::Shell::Fish),
            Generated::Elvish => completions(clap_complete::Shell::Elvish),
            Generated::Powershell => completions(clap_complete::Shell::PowerShell),
        },
        Command::Lsp => sprig_lsp::stdio(&args::clock(&cli))
            .map(|()| ExitCode::SUCCESS)
            .map_err(|e| e.to_string().into()),
        Command::Mcp(a) if a.config => data::print_json(&mcp::config(&a.plans)).map(|()| {
            eprintln!(
                "Add this to your agent's MCP configuration, such as .mcp.json in the project."
            );
            ExitCode::SUCCESS
        }),
        Command::Mcp(a) if a.instructions => agent::instructions(a.write.as_deref()),
        Command::Mcp(a) => mcp::stdio(a.plans.clone(), &args::clock(&cli), a.max_new_items)
            .map(|()| ExitCode::SUCCESS),
        Command::Tui(a) => tui::tui(a, &args::clock(&cli)),
        Command::Complete(a) => views::candidates(&a.what, today),
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("sprig: {e}");
            ExitCode::from(2)
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

/// The completion script clap writes from the argument definitions, with refs
/// completed from the plans in the current folder where the shell allows it.
fn completions(shell: clap_complete::Shell) -> Result<ExitCode> {
    let mut script = Vec::new();
    clap_complete::generate(shell, &mut Cli::command(), "sprig", &mut script);
    let mut script = String::from_utf8(script)?;
    // The scripts are clap's; these patches assume its layout, and a test
    // fails if a new clap moves it.
    match shell {
        clap_complete::Shell::Bash => {
            // The words offered after `tick` and `why` gain the folder's anchors.
            for cmd in ["sprig__subcmd__tick", "sprig__subcmd__why"] {
                let marker = format!("{cmd})\n            opts=\"");
                if let Some(at) = script.find(&marker) {
                    let end = script[at + marker.len()..]
                        .find('"')
                        .map(|e| at + marker.len() + e);
                    if let Some(end) = end {
                        script.insert_str(end, " $(sprig __complete refs 2>/dev/null)");
                    }
                }
            }
        }
        clap_complete::Shell::Zsh => {
            script = script
                .replace(
                    "':item -- The item to explain:_default'",
                    "':item -- The item to explain:_sprig_refs'",
                )
                .replace(
                    "'*::refs -- Items to tick:_default'",
                    "'*::refs -- Items to tick:_sprig_refs'",
                );
            // Defined before clap's dispatch at the end of the file: when zsh
            // autoloads the file from fpath, that dispatch runs the completion
            // at once, and a helper defined after it would not exist yet.
            let helper = "_sprig_refs() {\n    local -a refs\n    refs=(${(f)\"$(sprig __complete refs 2>/dev/null)\"})\n    _describe 'ref' refs\n}\n\n";
            let dispatch = "if [ \"$funcstack[1]\" = \"_sprig\" ]";
            match script.find(dispatch) {
                Some(at) => script.insert_str(at, helper),
                None => script.push_str(helper),
            }
        }
        _ => {}
    }
    io::stdout().lock().write_all(script.as_bytes())?;
    Ok(ExitCode::SUCCESS)
}

/// The manual page, with a section for each command.
fn man() -> Result<ExitCode> {
    let cmd = Cli::command();
    let mut w = io::stdout().lock();
    let page = clap_mangen::Man::new(cmd.clone());
    page.render_title(&mut w)?;
    page.render_name_section(&mut w)?;
    page.render_synopsis_section(&mut w)?;
    page.render_description_section(&mut w)?;
    page.render_options_section(&mut w)?;
    page.render_subcommands_section(&mut w)?;
    for sub in cmd.get_subcommands().filter(|s| !s.is_hide_set()) {
        writeln!(w, ".SH \"SPRIG {}\"", sub.get_name().to_uppercase())?;
        let mut help = Vec::new();
        sub.clone()
            .bin_name(format!("sprig {}", sub.get_name()))
            .write_long_help(&mut help)?;
        writeln!(w, ".nf")?;
        for line in String::from_utf8_lossy(&help).lines() {
            // A roff control character at the start of a line must be escaped.
            let line = line.replace('\\', "\\e");
            let line = if line.starts_with(['.', '\'']) {
                format!("\\&{line}")
            } else {
                line
            };
            writeln!(w, "{line}")?;
        }
        writeln!(w, ".fi")?;
    }
    page.render_extra_section(&mut w)?;
    page.render_version_section(&mut w)?;
    page.render_authors_section(&mut w)?;
    Ok(ExitCode::SUCCESS)
}
