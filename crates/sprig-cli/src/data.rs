//! The commands that read plans as data or turn other data into plans:
//! `find`, `agenda`, `graph`, `export` and `import`, and the JSON every
//! `--json` prints. The MCP server returns the same values, built by the
//! same functions, so a tool and a command cannot disagree.

use std::io::{self, Read, Write};
use std::process::ExitCode;

use serde_json::{Value, json};
use sprig_core::next::{self, Entry};
use sprig_core::workspace::{Resolution, Resolved, Tree};
use sprig_core::{Date, Severity, agenda, export, graph, import, json as tree, query};

use crate::args::{
    AgendaArgs, ExportArgs, ExportFormat, FindArgs, GraphArgs, GraphFormat, ImportArgs,
    ImportFormat,
};
use crate::load::Loaded;
use crate::out::{Cell, Out, Row, Style, Table};
use crate::views::{all_diagnostics, entry_trees, glyph, key, relative, resolve_all};
use crate::{Result, load};

/// One item as `--json` gives it (spec/tools.md T5): its ref, the labels
/// above it, and the node in the tree format, without its children.
pub fn item_json(n: &Resolved, path: &[&str]) -> Value {
    let mut node = tree::resolved(n);
    if let Some(m) = node.as_object_mut() {
        m.remove("children");
    }
    json!({ "ref": key(n), "path": path, "node": node })
}

/// `next --json`: the ready work, as `next` lists it.
pub fn next_json(
    groups: &[Loaded],
    resolved: &[Resolution],
    who: Option<&str>,
    limit: Option<usize>,
) -> Value {
    let trees: Vec<&Tree> = groups
        .iter()
        .zip(resolved)
        .flat_map(|(g, r)| entry_trees(g, r))
        .collect();
    let n = next::next(&trees, who);
    let shown: Vec<&Entry> = n.ready.iter().take(limit.unwrap_or(usize::MAX)).collect();
    Value::Array(shown.iter().map(|e| item_json(e.node, &e.path)).collect())
}

/// `find --json`.
pub fn find_json(resolved: &[Resolution], q: &query::Query) -> Value {
    let hits: Vec<query::Hit> = resolved.iter().flat_map(|r| query::find(r, q)).collect();
    Value::Array(hits.iter().map(|h| item_json(h.node, &h.path)).collect())
}

/// `status --json`: a row per plan, with the counts `status` shows.
pub fn status_json(groups: &[Loaded], resolved: &[Resolution], today: Date) -> Value {
    let mut rows = Vec::new();
    for (g, r) in groups.iter().zip(resolved) {
        for t in r.trees() {
            if g.entries.as_ref().is_some_and(|e| !e.contains(&t.file)) {
                continue;
            }
            let n = next::next(&[t], None);
            let late = n
                .ready
                .iter()
                .chain(&n.waiting)
                .filter(|e| e.due_date.is_some_and(|d| d < today))
                .count();
            let errors = r
                .diagnostics
                .iter()
                .filter(|d| d.file == t.file && d.severity() == Severity::Error)
                .count();
            let p = t.root.progress;
            rows.push(json!({
                "file": g.display(&t.file),
                "title": t.root.label,
                "done": p.done,
                "total": p.total,
                "ready": n.ready.len(),
                "waiting": n.waiting.len(),
                "late": late,
                "errors": errors,
            }));
        }
    }
    Value::Array(rows)
}

/// `check --json`: every diagnostic in the tree format's shape, in the order
/// `check` prints them.
pub fn check_json(groups: &[Loaded], resolved: &[Resolution]) -> Value {
    tree::diagnostics(&all_diagnostics(groups, resolved))
}

pub(crate) fn print_json(v: &Value) -> Result<()> {
    let mut w = io::stdout().lock();
    serde_json::to_writer_pretty(&mut w, v)?;
    writeln!(w)?;
    Ok(())
}

// ----------------------------------------------------------------------------
// find

pub fn find(a: &FindArgs, today: Date, out: Out) -> Result<ExitCode> {
    let q = match query::parse(&a.query.join(" "), today) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("sprig: {e}");
            return Ok(ExitCode::from(2));
        }
    };
    let (_groups, resolved) = resolve_all(std::slice::from_ref(&a.plans), today)?;
    let hits: Vec<query::Hit> = resolved.iter().flat_map(|r| query::find(r, &q)).collect();
    if a.json {
        print_json(&find_json(&resolved, &q))?;
    } else {
        let rows = hits
            .iter()
            .map(|h| {
                let n = h.node;
                let mark = glyph(n);
                let due = n
                    .due
                    .map(|d| {
                        if out.plain {
                            d.to_string()
                        } else {
                            format!("due {d} ({})", relative(today.days_until(d)))
                        }
                    })
                    .unwrap_or_default();
                let mut cells = vec![
                    vec![Cell::new(mark.to_string(), Style::Mark(mark))],
                    vec![Cell::plain(n.label.clone())],
                    vec![Cell::new(h.path.join(" › "), Style::Dim)],
                    vec![Cell::plain(due)],
                ];
                if out.plain {
                    let state = if n.blocked && n.status.is_open() {
                        "blocked"
                    } else {
                        n.status.as_str()
                    };
                    cells.push(vec![Cell::plain(state)]);
                } else {
                    cells.insert(0, vec![Cell::new(key(n), Style::Dim)]);
                }
                Row { key: key(n), cells }
            })
            .collect();
        let table = Table {
            rows,
            shrink: vec![(3, 6), (2, 10), (0, 12)],
            keep_end: vec![3],
        };
        let mut w = io::stdout().lock();
        for line in out.table(&table) {
            writeln!(w, "{line}")?;
        }
    }
    Ok(if hits.is_empty() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

// ----------------------------------------------------------------------------
// agenda

const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// `Thu 1 Oct`.
fn day(d: Date) -> String {
    let iso = d.to_string();
    let month: usize = iso[5..7].parse().expect("an ISO date");
    let dom: u32 = iso[8..10].parse().expect("an ISO date");
    format!(
        "{} {dom} {}",
        DAYS[usize::from(d.weekday())],
        MONTHS[month - 1]
    )
}

pub fn agenda(a: &AgendaArgs, today: Date, out: Out) -> Result<ExitCode> {
    let (_groups, resolved) = resolve_all(&a.paths, today)?;
    let items: Vec<agenda::Dated> = resolved.iter().flat_map(agenda::dated).collect();
    let w = agenda::window(items, today, i64::from(a.days));
    let row = |d: &agenda::Dated| {
        let n = d.node;
        let mark = glyph(n);
        let every = d.every.map(|e| format!("every:{e}")).unwrap_or_default();
        if out.plain {
            return Row {
                key: key(n),
                cells: vec![
                    vec![Cell::plain(d.due.to_string())],
                    vec![Cell::plain(mark.to_string())],
                    vec![Cell::plain(n.label.clone())],
                    vec![Cell::plain(every)],
                ],
            };
        }
        let late = today.days_until(d.due);
        Row {
            key: key(n),
            cells: vec![
                vec![
                    Cell::plain("  "),
                    Cell::new(mark.to_string(), Style::Mark(mark)),
                    Cell::plain(format!(" {}", n.label)),
                ],
                vec![Cell::new(
                    if late < 0 {
                        relative(late)
                    } else {
                        String::new()
                    },
                    Style::Warning,
                )],
                vec![Cell::new(every, Style::Dim)],
                vec![Cell::new(format!("{}:{}", n.file, n.line), Style::Dim)],
            ],
        }
    };
    let mut lines = Vec::new();
    let mut sections: Vec<(String, Vec<Row>)> = Vec::new();
    if !w.late.is_empty() {
        sections.push(("Late".into(), w.late.iter().map(row).collect()));
    }
    for (d, items) in &w.days {
        let name = match today.days_until(*d) {
            0 => format!("{} · today", day(*d)),
            1 => format!("{} · tomorrow", day(*d)),
            _ => day(*d),
        };
        sections.push((name, items.iter().map(row).collect()));
    }
    if out.plain {
        let rows: Vec<Row> = sections.into_iter().flat_map(|(_, r)| r).collect();
        lines.extend(out.table(&Table {
            rows,
            shrink: vec![],
            keep_end: vec![],
        }));
    } else {
        // One table under all the headings, so columns line up across days.
        let sizes: Vec<(String, usize)> =
            sections.iter().map(|(n, r)| (n.clone(), r.len())).collect();
        let rows: Vec<Row> = sections.into_iter().flat_map(|(_, r)| r).collect();
        let mut body = out
            .table(&Table {
                rows,
                shrink: vec![(0, 12), (3, 8)],
                keep_end: vec![3],
            })
            .into_iter();
        for (i, (name, n)) in sizes.into_iter().enumerate() {
            if i > 0 {
                lines.push(String::new());
            }
            lines.push(out.paint(&name, Style::Bold));
            lines.extend(body.by_ref().take(n));
        }
        if lines.is_empty() {
            lines.push(format!("Nothing is due in the next {} days", a.days));
        }
    }
    let mut out_w = io::stdout().lock();
    for l in lines {
        writeln!(out_w, "{l}")?;
    }
    Ok(ExitCode::SUCCESS)
}

// ----------------------------------------------------------------------------
// graph and export

pub fn graph(a: &GraphArgs, today: Date) -> Result<ExitCode> {
    let (_groups, resolved) = resolve_all(&a.paths, today)?;
    let mut w = io::stdout().lock();
    for r in &resolved {
        let text = match a.format {
            GraphFormat::Dot => graph::dot(r),
            GraphFormat::Mermaid => graph::mermaid(r),
        };
        w.write_all(text.as_bytes())?;
    }
    Ok(ExitCode::SUCCESS)
}

pub fn export(a: &ExportArgs, today: Date) -> Result<ExitCode> {
    let (groups, resolved) = resolve_all(&a.paths, today)?;
    let mut w = io::stdout().lock();
    match a.format {
        ExportFormat::Json => {
            let docs: Vec<Value> = groups
                .iter()
                .zip(&resolved)
                .flat_map(|(g, r)| {
                    entry_trees(g, r)
                        .into_iter()
                        .filter_map(|t| tree::tree_document(r, &t.file, today))
                        .collect::<Vec<_>>()
                })
                .collect();
            drop(w);
            print_json(&Value::Array(docs))?;
        }
        ExportFormat::Ics => {
            let items: Vec<agenda::Dated> = resolved.iter().flat_map(agenda::dated).collect();
            w.write_all(export::ics(&items, today).as_bytes())?;
        }
        ExportFormat::Csv => {
            for (i, r) in resolved.iter().enumerate() {
                let csv = export::csv(r);
                // One header for the lot.
                let body = if i == 0 {
                    csv.as_str()
                } else {
                    csv.split_once("\r\n").map_or("", |(_, b)| b)
                };
                w.write_all(body.as_bytes())?;
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

// ----------------------------------------------------------------------------
// import

pub fn import(a: &ImportArgs) -> Result<ExitCode> {
    let stdin = a.file.as_os_str() == "-";
    let src = if stdin {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s)?;
        s
    } else {
        std::fs::read_to_string(&a.file).map_err(|error| load::LoadError {
            path: a.file.clone(),
            error,
        })?
    };
    let ext = a
        .file
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let from = match a.from.or(match ext.as_str() {
        "md" | "markdown" => Some(ImportFormat::Markdown),
        "txt" => Some(ImportFormat::Todotxt),
        "json" => Some(ImportFormat::Github),
        _ => None,
    }) {
        Some(f) => f,
        None => {
            eprintln!(
                "sprig: cannot tell the format of {}; name it with --from markdown, todotxt or github",
                a.file.display()
            );
            return Ok(ExitCode::from(2));
        }
    };
    let stem = a
        .file
        .file_stem()
        .filter(|_| !stdin)
        .map(|s| s.to_string_lossy().into_owned());
    let title = a.title.clone().unwrap_or_else(|| match from {
        ImportFormat::Github => "GitHub issues".into(),
        _ => stem.unwrap_or_else(|| "Imported".into()),
    });
    let text = match from {
        ImportFormat::Markdown => import::markdown(&src, &title),
        ImportFormat::Todotxt => import::todotxt(&src, &title),
        ImportFormat::Github => match import::github(&src, &title) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("sprig: {}: {e}", a.file.display());
                return Ok(ExitCode::from(2));
            }
        },
    };
    io::stdout().lock().write_all(text.as_bytes())?;
    Ok(ExitCode::SUCCESS)
}
