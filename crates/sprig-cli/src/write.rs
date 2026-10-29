//! The commands that change files, `fmt` and `tick`, and what the MCP server
//! and the TUI share with them.
//!
//! Everything else that changes a plan is a change to its text, made in an
//! editor: the file is the truth (cairn 0154). Every change here comes from
//! the core's edit API, which touches only the bytes it means to; this module
//! finds the files, decides what to ask for, and writes the answer back. A
//! file is replaced in one step, by writing beside it and renaming, so an
//! interrupted write never leaves half a plan.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use sprig_core::workspace::Resolved;
use sprig_core::{Date, Resolution, edit, refs, syntax};

use crate::args::{FmtArgs, TickArgs};
use crate::load::{self, Loaded};
use crate::views::wait_label;
use crate::{Result, plural};

/// Replaces a file's contents in one step.
pub(crate) fn save(path: &Path, text: &str) -> io::Result<()> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = path.with_file_name(format!(".{name}.sprig-{}", std::process::id()));
    fs::write(&tmp, text)?;
    fs::rename(&tmp, path).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

pub(crate) fn write_error(path: &Path, e: io::Error) -> Box<dyn std::error::Error> {
    format!("cannot write {}: {e}", path.display()).into()
}

/// The files a command was pointed at, each with where it lives on disk: a
/// named file, or every file under a named directory, but not files that are
/// only read because something grafts them.
fn targets(g: &Loaded) -> Vec<(String, PathBuf, &str)> {
    g.workspace
        .files()
        .filter(|(key, _)| {
            g.entries
                .as_ref()
                .is_none_or(|e| e.iter().any(|k| k == key))
        })
        .map(|(key, src)| (g.display(key), g.root.join(key), src))
        .collect()
}

pub fn fmt(a: &FmtArgs, today: Date) -> Result<ExitCode> {
    let groups = load::load(&a.paths)?;
    let mut changed = Vec::new();
    let mut total = 0;
    for g in &groups {
        for (shown, path, src) in targets(g) {
            total += 1;
            let c = edit::format(src, today);
            if c.is_empty() {
                continue;
            }
            if !a.check {
                save(&path, &c.source).map_err(|e| write_error(&path, e))?;
            }
            changed.push(shown);
        }
    }
    let mut out = io::stdout().lock();
    if a.check {
        // One path per line, as `gofmt -l` prints them, so the list pipes.
        for f in &changed {
            writeln!(out, "{f}")?;
        }
        eprintln!(
            "{} of {} would be reformatted",
            changed.len(),
            plural(total, "file")
        );
        return Ok(if changed.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        });
    }
    eprintln!("reformatted {} of {}", changed.len(), plural(total, "file"));
    Ok(ExitCode::SUCCESS)
}

pub fn tick(a: &TickArgs, today: Date) -> Result<ExitCode> {
    // `-` stands for the refs on standard input, one per line.
    let mut refs: Vec<String> = Vec::new();
    for r in &a.refs {
        if r == "-" {
            for line in io::stdin().lock().lines() {
                let line = line?;
                if !line.trim().is_empty() {
                    refs.push(line);
                }
            }
        } else {
            refs.push(r.clone());
        }
    }

    let mut groups = load::load(std::slice::from_ref(&a.plans))?;
    let g = groups.pop().expect("one path gives one workspace");
    let before = g.workspace.resolve(today);

    // Every ref is resolved before anything is written: all or nothing.
    let mut items: Vec<refs::Item> = Vec::new();
    let mut failed = false;
    for r in &refs {
        match refs::resolve(&before, r) {
            Ok(i) if !items.contains(&i) => items.push(i),
            Ok(_) => {}
            Err(e) => {
                eprintln!("sprig: {e}");
                failed = true;
            }
        }
    }
    if failed {
        eprintln!("sprig: nothing was ticked");
        return Ok(ExitCode::from(2));
    }

    // A tick or a mark changes no line's number, so they apply one after
    // another, and every one is made before any file is written.
    let mut by_file: BTreeMap<&str, Vec<(usize, &refs::Item)>> = BTreeMap::new();
    for (n, i) in items.iter().enumerate() {
        by_file.entry(i.file.as_str()).or_default().push((n, i));
    }
    let mut after = g.workspace.clone();
    let mut writes = Vec::new();
    let mut report = Vec::new();
    for (file, items) in by_file {
        let (_, src) = g
            .workspace
            .files()
            .find(|(k, _)| *k == file)
            .expect("a resolved file is loaded");
        let mut text = src.to_string();
        for (n, i) in items {
            let (change, moved) = match a.mark {
                Some(m) => (edit::set_mark(&text, i.line, m.0), Ok(None)),
                None => match edit::tick(&text, i.line, today) {
                    Ok((c, moved)) => (Ok(c), Ok(moved)),
                    Err(e) => (Err(e), Err(())),
                },
            };
            let change = match change {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("sprig: {}: {e}", g.display(file));
                    eprintln!("sprig: nothing was ticked");
                    return Ok(ExitCode::from(2));
                }
            };
            text = change.source;
            let line = text.lines().nth(i.line - 1).unwrap_or("").trim();
            let note = moved
                .ok()
                .flatten()
                .map(|d| format!("  (due moved to {d})"))
                .unwrap_or_default();
            report.push((n, format!("{}:{}  {line}{note}", g.display(file), i.line)));
        }
        after.insert(file.to_string(), text.clone());
        writes.push((g.root.join(file), text));
    }
    for (path, text) in &writes {
        save(path, text).map_err(|e| write_error(path, e))?;
    }
    // A line per ref, in the order they were given, so the output pipes on.
    report.sort_by_key(|(n, _)| *n);
    let mut out = io::stdout().lock();
    for (_, r) in report {
        writeln!(out, "{r}")?;
    }
    // What the change unblocked is for the person, so it goes where git puts
    // its notes, and leaves standard output a line per ref.
    let after = after.resolve(today);
    let freed = freed(&before, &after, |_, l| l);
    if !freed.is_empty() {
        let mut err = io::stderr().lock();
        writeln!(err, "No longer waiting on it:")?;
        for f in freed {
            let n = f.node;
            writeln!(
                err,
                "  {}:{}  {}  ({})",
                g.display(&n.file),
                n.line,
                n.label,
                f.rest()
            )?;
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// An open item that waits on less after a change than before it.
pub(crate) struct Freed<'r> {
    pub node: &'r Resolved,
    /// What it still waits on; empty when it is unblocked.
    pub still: Vec<String>,
}

impl Freed<'_> {
    /// `unblocked`, or what it still waits on. An item freed of one of two
    /// waits is listed too, since that is what the change did for it; calling
    /// it unblocked would say more than the file does.
    pub fn rest(&self) -> String {
        if self.still.is_empty() && !self.node.blocked {
            "unblocked".to_string()
        } else if self.still.is_empty() {
            "still blocked above it".to_string()
        } else {
            format!("still waits on {}", self.still.join(", "))
        }
    }
}

/// The open items that wait on less after a change than before it. `moved`
/// maps a line before the change to the same line after it.
pub(crate) fn freed<'r>(
    before: &Resolution,
    after: &'r Resolution,
    moved: impl Fn(&str, usize) -> usize,
) -> Vec<Freed<'r>> {
    fn walk<'r>(n: &'r Resolved, out: &mut Vec<&'r Resolved>) {
        for c in n.children.iter().filter(|c| !c.grafted) {
            out.push(c);
            walk(c, out);
        }
    }
    fn open(r: &Resolution) -> Vec<&Resolved> {
        let mut all = Vec::new();
        for t in r.trees() {
            walk(&t.root, &mut all);
        }
        all.retain(|n| n.status.is_open());
        all
    }
    let was: std::collections::HashMap<(String, usize), usize> = open(before)
        .into_iter()
        .map(|n| ((n.file.clone(), moved(&n.file, n.line)), n.waits_on.len()))
        .collect();
    let mut freed: Vec<&Resolved> = open(after)
        .into_iter()
        .filter(|n| {
            was.get(&(n.file.clone(), n.line))
                .is_some_and(|&w| n.waits_on.len() < w)
        })
        .collect();
    freed.dedup_by_key(|n| (n.file.clone(), n.line));
    freed
        .into_iter()
        .map(|node| Freed {
            node,
            still: node.waits_on.iter().map(wait_label).collect(),
        })
        .collect()
}

/// An item's line as typed: without a mark it is a to-do, and typed dates
/// are written out (§9.4).
pub(crate) fn item_line(text: &str, today: Date) -> String {
    let text = text.trim();
    let marked = syntax::lex(&format!("_\n{text}"))
        .get(1)
        .is_some_and(|l| matches!(l.kind, syntax::LineKind::Item(_)));
    let text = if marked {
        text.to_string()
    } else {
        format!("- {text}")
    };
    syntax::expand_dates(&text, today)
}

/// `src` with `text` as a new last line, in the file's own line ending.
pub(crate) fn appended(src: &str, text: &str) -> String {
    let eol = if src.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out = src.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push_str(eol);
    }
    out.push_str(text);
    out.push_str(eol);
    out
}

/// Opens a file at a line in the person's editor and waits for it: $VISUAL,
/// then $EDITOR, then vi, as git and crontab choose. The value may carry
/// arguments, as in `code --wait`. Says whether the editor succeeded.
pub(crate) fn run_editor(path: &Path, line: usize) -> Result<bool> {
    let editor = ["VISUAL", "EDITOR"]
        .iter()
        .find_map(|v| std::env::var(v).ok().filter(|e| !e.trim().is_empty()))
        .unwrap_or_else(|| "vi".into());
    let mut words = editor.split_whitespace();
    let program = words.next().expect("the editor is not empty");
    let status = Command::new(program)
        .args(words)
        .arg(format!("+{line}"))
        .arg(path)
        .status()
        .map_err(|e| format!("cannot run {program}: {e}"))?;
    Ok(status.success())
}
