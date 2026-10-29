//! The write verbs: `mark`, `answer`, `add` and `edit`.
//!
//! Each resolves one ref, asks the core's edit API for the change (the same
//! calls the language server and the MCP server make), and writes the file
//! back in one step. Nothing is written when the ref is unknown or
//! ambiguous, or when the item cannot take the change.

use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::{Command, ExitCode};

use sprig_core::edit::{self, Change, Place};
use sprig_core::workspace::{Resolved, Wait};
use sprig_core::{Date, Resolution, Workspace, refs, syntax};

use crate::Result;
use crate::args::{AddArgs, AnswerArgs, EditArgs, MarkArgs};
use crate::load::{self, Loaded};
use crate::write::{save, write_error};

/// The workspace a write acts on, and the one item a ref names in it.
fn locate(
    plans: &Path,
    r: &str,
    today: Date,
) -> Result<std::result::Result<(Loaded, refs::Item), ExitCode>> {
    let mut groups = load::load(std::slice::from_ref(&plans.to_path_buf()))?;
    let g = groups.pop().expect("one path gives one workspace");
    let res = g.workspace.resolve(today);
    match refs::resolve(&res, r) {
        Ok(i) => Ok(Ok((g, i))),
        Err(e) => {
            eprintln!("sprig: {e}");
            eprintln!("sprig: nothing was written");
            Ok(Err(ExitCode::from(2)))
        }
    }
}

fn source<'g>(g: &'g Loaded, file: &str) -> &'g str {
    g.workspace
        .files()
        .find(|(k, _)| *k == file)
        .map(|(_, s)| s)
        .expect("a resolved file is loaded")
}

/// The 1-based line a byte offset falls on.
fn line_at(src: &str, offset: usize) -> usize {
    src[..offset].matches('\n').count() + 1
}

/// Writes a change and prints the line it is about, as `file:line  text`.
fn commit(g: &Loaded, file: &str, change: &Change, line: usize) -> Result<()> {
    let path = g.root.join(file);
    save(&path, &change.source).map_err(|e| write_error(&path, e))?;
    let text = change.source.lines().nth(line - 1).unwrap_or("").trim();
    writeln!(io::stdout().lock(), "{}:{line}  {text}", g.display(file))?;
    Ok(())
}

pub fn mark(a: &MarkArgs, today: Date) -> Result<ExitCode> {
    let (g, item) = match locate(&a.plans, &a.item, today)? {
        Ok(found) => found,
        Err(code) => return Ok(code),
    };
    match edit::set_mark(source(&g, &item.file), item.line, a.mark.0) {
        Ok(change) => {
            let after = resolve_with(&g, &item.file, &change, today);
            commit(&g, &item.file, &change, item.line)?;
            report_freed(&g, &g.workspace.resolve(today), &after, |_, l| l)?;
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            eprintln!("sprig: {}: {e}", g.display(&item.file));
            Ok(ExitCode::from(2))
        }
    }
}

pub fn answer(a: &AnswerArgs, today: Date) -> Result<ExitCode> {
    let (g, item) = match locate(&a.plans, &a.item, today)? {
        Ok(found) => found,
        Err(code) => return Ok(code),
    };
    let src = source(&g, &item.file);
    let change = match edit::add_answer(src, item.line, &a.text) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("sprig: {}: {e}", g.display(&item.file));
            return Ok(ExitCode::from(2));
        }
    };
    let at = line_at(src, change.edits[0].span.start);
    let after = resolve_with(&g, &item.file, &change, today);
    commit(&g, &item.file, &change, at)?;
    // Lines at and below the new answer moved down one in its file.
    let moved = |file: &str, line: usize| {
        if file == item.file && line >= at {
            line + 1
        } else {
            line
        }
    };
    report_freed(&g, &g.workspace.resolve(today), &after, moved)?;
    Ok(ExitCode::SUCCESS)
}

/// The workspace as it will read once `change` is written to `file`.
fn resolve_with(g: &Loaded, file: &str, change: &Change, today: Date) -> Resolution {
    let mut ws: Workspace = g.workspace.clone();
    ws.insert(file.to_string(), change.source.clone());
    ws.resolve(today)
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
            still: node
                .waits_on
                .iter()
                .map(|w| match w {
                    Wait::Reference { label, .. } => label.clone(),
                    Wait::Start(d) => format!("its start, {d}"),
                })
                .collect(),
        })
        .collect()
}

/// Prints what [`freed`] finds, under a heading, after the changed line.
fn report_freed(
    g: &Loaded,
    before: &Resolution,
    after: &Resolution,
    moved: impl Fn(&str, usize) -> usize,
) -> Result<()> {
    let freed = freed(before, after, moved);
    if freed.is_empty() {
        return Ok(());
    }
    let mut out = io::stdout().lock();
    writeln!(out, "\nNo longer waiting on it:")?;
    for f in freed {
        let n = f.node;
        writeln!(
            out,
            "  {}:{}  {}  ({})",
            g.display(&n.file),
            n.line,
            n.label,
            f.rest()
        )?;
    }
    Ok(())
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

pub fn add(a: &AddArgs, today: Date) -> Result<ExitCode> {
    let text = item_line(&a.text, today);

    let (r, place) = match (&a.under, &a.after) {
        (Some(r), _) => (r, Place::Child),
        (None, Some(r)) => (r, Place::Sibling),
        (None, None) => return add_to_plan(a, &text),
    };
    let (g, item) = match locate(&a.plans, r, today)? {
        Ok(found) => found,
        Err(code) => return Ok(code),
    };
    let src = source(&g, &item.file);
    match edit::add_item(src, item.line, place, &text) {
        Ok(change) => {
            let at = line_at(src, change.edits[0].span.start);
            // The new line starts after the line ending inserted before it,
            // or at the start of the line the insertion point is on.
            let line = if change.edits[0].text.starts_with(['\n', '\r']) {
                at + 1
            } else {
                at
            };
            commit(&g, &item.file, &change, line)?;
            Ok(ExitCode::SUCCESS)
        }
        Err(e) => {
            eprintln!("sprig: {}: {e}", g.display(&item.file));
            Ok(ExitCode::from(2))
        }
    }
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

/// Without a parent, a new item goes at the end of the folder's PLAN.sprig,
/// the plan an agent reads first.
fn add_to_plan(a: &AddArgs, text: &str) -> Result<ExitCode> {
    let path = a.plans.join("PLAN.sprig");
    let src = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!(
                "sprig: no PLAN.sprig in {}; name a parent with --under or --after",
                a.plans.display()
            );
            return Ok(ExitCode::from(2));
        }
    };
    let out = appended(&src, text);
    save(&path, &out).map_err(|e| write_error(&path, e))?;
    let line = out.lines().count();
    let shown = if a.plans == Path::new(".") {
        "PLAN.sprig".to_string()
    } else {
        path.display().to_string()
    };
    writeln!(io::stdout().lock(), "{shown}:{line}  {text}")?;
    Ok(ExitCode::SUCCESS)
}

pub fn edit(a: &EditArgs, today: Date) -> Result<ExitCode> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        eprintln!(
            "sprig: edit opens an editor, which needs a terminal; `sprig why` or `sprig tree` print the item instead"
        );
        return Ok(ExitCode::from(2));
    }
    let (g, item) = match locate(&a.plans, &a.item, today)? {
        Ok(found) => found,
        Err(code) => return Ok(code),
    };
    // $VISUAL, then $EDITOR, then vi, as git and crontab choose; the value may
    // carry arguments, as in `code --wait`.
    let editor = ["VISUAL", "EDITOR"]
        .iter()
        .find_map(|v| std::env::var(v).ok().filter(|e| !e.trim().is_empty()))
        .unwrap_or_else(|| "vi".into());
    let mut words = editor.split_whitespace();
    let program = words.next().expect("the editor is not empty");
    let status = Command::new(program)
        .args(words)
        .arg(format!("+{}", item.line))
        .arg(g.root.join(&item.file))
        .status()
        .map_err(|e| format!("cannot run {program}: {e}"))?;
    Ok(if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    })
}
