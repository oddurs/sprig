//! The commands that change files: `fmt` and `tick`.
//!
//! Every change comes from the core's edit API, which touches only the bytes
//! it means to; this module finds the files, decides what to ask for, and
//! writes the answer back. A file is replaced in one step, by writing beside
//! it and renaming, so an interrupted write never leaves half a plan.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sprig_core::{Date, edit, refs};

use crate::args::{FmtArgs, TickArgs};
use crate::load::{self, Loaded};
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
    let res = g.workspace.resolve(today);

    // Every ref is resolved before anything is written: all or nothing.
    let mut items: Vec<refs::Item> = Vec::new();
    let mut failed = false;
    for r in &refs {
        match refs::resolve(&res, r) {
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

    // Ticks change no line's number, so they apply one after another.
    let mut by_file: BTreeMap<&str, Vec<(usize, &refs::Item)>> = BTreeMap::new();
    for (n, i) in items.iter().enumerate() {
        by_file.entry(i.file.as_str()).or_default().push((n, i));
    }
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
            let (change, moved) = edit::tick(&text, i.line, today)
                .map_err(|e| format!("{}: {e}", g.display(file)))?;
            text = change.source;
            let line = text.lines().nth(i.line - 1).unwrap_or("").trim();
            let note = moved
                .map(|d| format!("  (due moved to {d})"))
                .unwrap_or_default();
            report.push((n, format!("{}:{}  {line}{note}", g.display(file), i.line)));
        }
        writes.push((g.root.join(file), text));
    }
    for (path, text) in &writes {
        save(path, text).map_err(|e| write_error(path, e))?;
    }
    // In the order the refs were given.
    report.sort_by_key(|(n, _)| *n);
    let mut out = io::stdout().lock();
    for (_, r) in report {
        writeln!(out, "{r}")?;
    }
    Ok(ExitCode::SUCCESS)
}
