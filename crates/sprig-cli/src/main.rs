use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;
use sprig_core::next::Entry;
use sprig_core::{Date, Diagnostic, Severity, Status, Tree, next};

mod args;
mod load;
mod write;

use args::{CheckArgs, Cli, Command, NextArgs};
use load::Loaded;

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

fn run() -> ExitCode {
    let cli = Cli::parse();
    let today = args::today(&cli);
    let result = match &cli.command {
        Command::Next(a) => cmd_next(a, today),
        Command::Check(a) => cmd_check(a, today),
        Command::Fmt(a) => write::fmt(a, today),
        Command::Tick(a) => write::tick(a, today),
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

/// Diagnostics with each file shown as the person would type it.
fn diagnostics(loaded: &Loaded, ds: &[Diagnostic]) -> Vec<Diagnostic> {
    ds.iter()
        .map(|d| Diagnostic {
            file: loaded.display(&d.file),
            ..d.clone()
        })
        .collect()
}

fn cmd_check(a: &CheckArgs, today: Date) -> Result<ExitCode> {
    let groups = load::load(&a.paths)?;
    let mut all = Vec::new();
    let mut files = 0;
    for g in &groups {
        let r = g.workspace.resolve(today);
        files += r.parsed.len();
        all.extend(diagnostics(g, &r.diagnostics));
    }
    all.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    let mut out = io::stdout().lock();
    for d in &all {
        writeln!(out, "{d}")?;
    }
    let errors = all
        .iter()
        .filter(|d| d.severity() == Severity::Error)
        .count();
    let warnings = all.len() - errors;
    let mut with: Vec<&str> = all.iter().map(|d| d.file.as_str()).collect();
    with.dedup();
    let files_word = |n: usize| if n == 1 { "file" } else { "files" };
    if all.is_empty() {
        eprintln!("{files} {} checked, no problems", files_word(files));
    } else {
        let mut parts = Vec::new();
        if errors > 0 {
            parts.push(plural(errors, "error"));
        }
        if warnings > 0 {
            parts.push(plural(warnings, "warning"));
        }
        eprintln!(
            "{} in {} of {files} {}",
            parts.join(" and "),
            with.len(),
            files_word(files)
        );
    }
    let failed = errors > 0 || (a.strict && warnings > 0);
    Ok(if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

pub(crate) fn plural(n: usize, word: &str) -> String {
    if n == 1 {
        format!("1 {word}")
    } else {
        format!("{n} {word}s")
    }
}

fn cmd_next(a: &NextArgs, today: Date) -> Result<ExitCode> {
    let groups = load::load(&a.paths)?;
    let who = a.who.as_deref().map(|w| w.strip_prefix('@').unwrap_or(w));
    let resolved: Vec<_> = groups.iter().map(|g| g.workspace.resolve(today)).collect();

    let mut trees: Vec<&Tree> = Vec::new();
    let mut errors = 0;
    for (g, r) in groups.iter().zip(&resolved) {
        errors += r
            .diagnostics
            .iter()
            .filter(|d| d.severity() == Severity::Error)
            .count();
        match &g.entries {
            Some(files) => trees.extend(files.iter().filter_map(|f| r.tree(f))),
            None => trees.extend(r.roots().into_iter().filter_map(|f| r.tree(f))),
        }
    }
    let n = next::next(&trees, who);

    let shown: Vec<&Entry> = n.ready.iter().take(a.limit.unwrap_or(usize::MAX)).collect();
    let mut out = io::stdout().lock();
    for line in table(&shown, today) {
        writeln!(out, "{line}")?;
    }
    let waiting = match n.waiting.len() {
        0 => String::new(),
        w => format!(" · {} waiting on something, not shown", w),
    };
    let questions = match n.questions() {
        0 => String::new(),
        q => format!(" · {}", plural(q, "open question")),
    };
    if n.ready.is_empty() {
        writeln!(out, "Nothing is ready{waiting}")?;
    } else {
        let ready = if shown.len() < n.ready.len() {
            format!("{} of {} ready", shown.len(), n.ready.len())
        } else {
            format!("{} ready", n.ready.len())
        };
        writeln!(out, "\n{ready}{waiting}{questions}")?;
    }
    if errors > 0 {
        eprintln!(
            "sprig: {} in these plans; `sprig check` lists {}",
            plural(errors, "error"),
            if errors == 1 { "it" } else { "them" }
        );
    }
    Ok(if n.ready.is_empty() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

/// One line per item: its mark and text, where it sits, when it is due, and
/// its priority, in columns.
fn table(entries: &[&Entry], today: Date) -> Vec<String> {
    const WIDE: usize = 40;
    let rows: Vec<[String; 4]> = entries
        .iter()
        .map(|e| {
            let mark = match e.node.status {
                Status::Doing => '~',
                Status::Question => '?',
                _ => '-',
            };
            let text = format!("{mark} {}", e.node.label);
            let path = if e.path.is_empty() {
                e.plan.to_string()
            } else {
                e.path.join(" › ")
            };
            let due = match (e.due, e.due_date) {
                (_, Some(d)) => format!("due {d} ({})", relative(today.days_until(d))),
                (Some(raw), None) => format!("due {raw}"),
                (None, None) => String::new(),
            };
            let priority = "!".repeat(usize::from(e.priority));
            [clip(&text, WIDE), clip_left(&path, WIDE), due, priority]
        })
        .collect();
    let width = |i: usize| rows.iter().map(|r| r[i].chars().count()).max().unwrap_or(0);
    let widths = [width(0), width(1), width(2)];
    rows.iter()
        .map(|r| {
            let mut line = String::new();
            for (i, cell) in r.iter().enumerate() {
                line.push_str(cell);
                if i < 3 {
                    let pad = widths[i] - cell.chars().count();
                    line.push_str(&" ".repeat(pad + 2));
                }
            }
            line.trim_end().to_string()
        })
        .collect()
}

fn relative(days: i64) -> String {
    match days {
        0 => "today".into(),
        d if d > 0 => format!("in {d}d"),
        d => format!("{}d late", -d),
    }
}

// A long path loses its top rather than its end: the nearest parent is the
// one that says where an item is.
fn clip_left(s: &str, max: usize) -> String {
    let n = s.chars().count();
    if n <= max {
        s.to_string()
    } else {
        let tail: String = s.chars().skip(n - (max - 1)).collect();
        format!("…{tail}")
    }
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max - 1).collect();
        out.push('…');
        out
    }
}
