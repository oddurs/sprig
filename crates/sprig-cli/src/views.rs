//! The commands that read and print: `next`, `check`, `tree`, `why` and
//! `status`. Each builds rows and leaves widths, colour and `--plain` to
//! [`crate::out`].

use std::collections::HashSet;
use std::io::{self, Write};
use std::process::ExitCode;

use sprig_core::next::{self, Entry};
use sprig_core::workspace::{Resolution, Resolved, Status, Tree, Wait};
use sprig_core::{Date, Diagnostic, Severity, refs};

use crate::args::{CheckArgs, NextArgs, StatusArgs, TreeArgs, WhyArgs};
use crate::load::{self, Loaded};
use crate::out::{Cell, Out, Row, Style, Table, clip};
use crate::{Result, plural};

/// The ref `--plain` puts first: an anchor where there is one, since it
/// survives edits, and the line otherwise.
pub(crate) fn key(n: &Resolved) -> String {
    match n.tokens.anchor() {
        Some(a) => format!("{}^{}", n.file, a.name),
        None => format!("{}:{}", n.file, n.line),
    }
}

/// The character a node shows: its mark, or, where an `x`, `/` or `>` above
/// settled it, the status that settled it (§5.2).
pub(crate) fn glyph(n: &Resolved) -> char {
    match n.mark {
        Some(m) if m.is_structure() => m.as_char(),
        _ if n.answered => '?',
        Some(m) if n.settled => match n.status {
            Status::Done => 'x',
            Status::Dropped => '/',
            Status::Later => '>',
            _ => m.as_char(),
        },
        Some(m) => m.as_char(),
        None => ' ',
    }
}

pub(crate) fn relative(days: i64) -> String {
    match days {
        0 => "today".into(),
        d if d > 0 => format!("in {d}d"),
        d => format!("{}d late", -d),
    }
}

fn hours(h: f64) -> String {
    let short = |x: f64| {
        let r = (x * 10.0).round() / 10.0;
        if r.fract() == 0.0 {
            format!("{r:.0}")
        } else {
            format!("{r:.1}")
        }
    };
    if h < 8.0 {
        format!("{}h", short(h))
    } else {
        format!("{}d", short(h / 8.0))
    }
}

pub(crate) fn resolve_all(
    paths: &[std::path::PathBuf],
    today: Date,
) -> Result<(Vec<Loaded>, Vec<Resolution>)> {
    let groups = load::load(paths)?;
    let resolved = groups.iter().map(|g| g.workspace.resolve(today)).collect();
    Ok((groups, resolved))
}

/// The trees a folder or a set of files stands for: the files named, or the
/// plans no file grafts whole.
pub(crate) fn entry_trees<'r>(g: &Loaded, r: &'r Resolution) -> Vec<&'r Tree> {
    match &g.entries {
        Some(files) => files.iter().filter_map(|f| r.tree(f)).collect(),
        None => r.roots().into_iter().filter_map(|f| r.tree(f)).collect(),
    }
}

fn errors_note(resolved: &[Resolution]) {
    let errors: usize = resolved
        .iter()
        .map(|r| {
            r.diagnostics
                .iter()
                .filter(|d| d.severity() == Severity::Error)
                .count()
        })
        .sum();
    if errors > 0 {
        eprintln!(
            "sprig: {} in these plans; `sprig check` lists {}",
            plural(errors, "error"),
            if errors == 1 { "it" } else { "them" }
        );
    }
}

// ----------------------------------------------------------------------------
// next

pub fn next(a: &NextArgs, today: Date, out: Out) -> Result<ExitCode> {
    let (groups, resolved) = resolve_all(&a.paths, today)?;
    let who = a.who.as_deref().map(|w| w.strip_prefix('@').unwrap_or(w));
    let trees: Vec<&Tree> = groups
        .iter()
        .zip(&resolved)
        .flat_map(|(g, r)| entry_trees(g, r))
        .collect();
    let n = next::next(&trees, who);
    let shown: Vec<&Entry> = n.ready.iter().take(a.limit.unwrap_or(usize::MAX)).collect();
    if a.json {
        let items: Vec<serde_json::Value> = shown
            .iter()
            .map(|e| crate::data::item_json(e.node, &e.path))
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(ExitCode::SUCCESS);
    }

    // Text and paths are capped even in a pipe, so a line stays a line.
    const WIDE: usize = 40;
    let rows = shown
        .iter()
        .map(|e| {
            let mark = match e.node.status {
                Status::Doing => '~',
                Status::Question => '?',
                _ => '-',
            };
            let path = if e.path.is_empty() {
                e.plan.to_string()
            } else {
                e.path.join(" › ")
            };
            if out.plain {
                let due = e
                    .due_date
                    .map(|d| d.to_string())
                    .or(e.due.map(String::from))
                    .unwrap_or_default();
                return Row {
                    key: key(e.node),
                    cells: vec![
                        vec![Cell::plain(mark.to_string())],
                        vec![Cell::plain(e.node.label.clone())],
                        vec![Cell::plain(path)],
                        vec![Cell::plain(due)],
                        vec![Cell::plain(e.priority.to_string())],
                    ],
                };
            }
            let due = match (e.due, e.due_date) {
                (_, Some(d)) => format!("due {d} ({})", relative(today.days_until(d))),
                (Some(raw), None) => format!("due {raw}"),
                (None, None) => String::new(),
            };
            let label = clip(&e.node.label, WIDE - 2);
            Row {
                key: key(e.node),
                cells: vec![
                    vec![
                        Cell::new(mark.to_string(), Style::Mark(mark)),
                        Cell::new(
                            format!(" {label}"),
                            if mark == '~' {
                                Style::Bold
                            } else {
                                Style::Plain
                            },
                        ),
                    ],
                    vec![Cell::new(crate::out::clip_left(&path, WIDE), Style::Dim)],
                    vec![Cell::plain(due)],
                    vec![Cell::new("!".repeat(usize::from(e.priority)), Style::Bold)],
                ],
            }
        })
        .collect();
    // The path gives way first, then the text, and the date last of all.
    let table = Table {
        rows,
        shrink: vec![(1, 4), (0, 8), (2, 10)],
        keep_end: vec![1],
    };
    let mut w = io::stdout().lock();
    for line in out.table(&table) {
        writeln!(w, "{line}")?;
    }
    if !out.plain {
        let waiting = match n.waiting.len() {
            0 => String::new(),
            w => format!(" · {w} waiting on something, not shown"),
        };
        let questions = match n.questions() {
            0 => String::new(),
            q => format!(" · {}", plural(q, "open question")),
        };
        if n.ready.is_empty() {
            writeln!(w, "Nothing is ready{waiting}")?;
        } else {
            let ready = if shown.len() < n.ready.len() {
                format!("{} of {} ready", shown.len(), n.ready.len())
            } else {
                format!("{} ready", n.ready.len())
            };
            writeln!(w, "\n{ready}{waiting}{questions}")?;
        }
    }
    errors_note(&resolved);
    Ok(if n.ready.is_empty() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

// ----------------------------------------------------------------------------
// check

/// Diagnostics with each file shown as the person would type it.
fn diagnostics(loaded: &Loaded, ds: &[Diagnostic]) -> Vec<Diagnostic> {
    ds.iter()
        .map(|d| Diagnostic {
            file: loaded.display(&d.file),
            ..d.clone()
        })
        .collect()
}

pub fn check(a: &CheckArgs, today: Date, out: Out) -> Result<ExitCode> {
    let (groups, resolved) = resolve_all(&a.paths, today)?;
    let mut all = Vec::new();
    let mut files = 0;
    for (g, r) in groups.iter().zip(&resolved) {
        files += r.parsed.len();
        all.extend(diagnostics(g, &r.diagnostics));
    }
    all.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    let mut w = io::stdout().lock();
    for d in &all {
        if out.plain {
            writeln!(
                w,
                "{}\t{}\t{}\t{}\t{}\t{}",
                d.file,
                d.line,
                d.col,
                d.severity().as_str(),
                d.code,
                d.message
            )?;
        } else {
            let sev = match d.severity() {
                Severity::Error => out.paint("error", Style::Error),
                Severity::Warning => out.paint("warning", Style::Warning),
            };
            writeln!(
                w,
                "{}:{}:{}: {sev}[{}]: {}",
                d.file, d.line, d.col, d.code, d.message
            )?;
        }
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

// ----------------------------------------------------------------------------
// tree

fn waits_text(waits: &[Wait]) -> String {
    let on: Vec<&str> = waits
        .iter()
        .filter_map(|w| match w {
            Wait::Reference { label, .. } => Some(label.as_str()),
            Wait::Start(_) => None,
        })
        .collect();
    let until: Vec<String> = waits
        .iter()
        .filter_map(|w| match w {
            Wait::Start(d) => Some(d.to_string()),
            Wait::Reference { .. } => None,
        })
        .collect();
    let mut parts = Vec::new();
    if !on.is_empty() {
        parts.push(format!("on {}", on.join(", ")));
    }
    if !until.is_empty() {
        parts.push(format!("until {}", until.join(", ")));
    }
    format!("waits {}", parts.join(" and "))
}

fn progress(n: &Resolved) -> String {
    if n.children.is_empty() || n.progress.total == 0 {
        return String::new();
    }
    let mut s = format!("{}/{}", n.progress.done, n.progress.total);
    if n.estimate > 0.0 {
        s.push_str(&format!(" · {} left", hours(n.estimate)));
    }
    s
}

pub fn tree(a: &TreeArgs, today: Date, out: Out) -> Result<ExitCode> {
    let (groups, resolved) = resolve_all(&a.paths, today)?;
    let mut w = io::stdout().lock();
    let mut first = true;
    for (g, r) in groups.iter().zip(&resolved) {
        for t in entry_trees(g, r) {
            if !first && !out.plain {
                writeln!(w)?;
            }
            first = false;
            let mut rows = vec![Row {
                key: format!("{}:{}", t.file, t.root.line),
                cells: if out.plain {
                    vec![
                        vec![Cell::plain("0")],
                        vec![Cell::plain("")],
                        vec![Cell::plain(t.root.label.clone())],
                        vec![Cell::plain(t.root.status.as_str())],
                        vec![Cell::plain(t.root.progress.done.to_string())],
                        vec![Cell::plain(t.root.progress.total.to_string())],
                    ]
                } else {
                    vec![
                        vec![Cell::new(t.root.label.clone(), Style::Bold)],
                        vec![Cell::new(g.display(&t.file), Style::Dim)],
                        vec![Cell::plain(progress(&t.root))],
                    ]
                },
            }];
            walk_tree(&t.root, "", 1, a, today, out, &mut rows);
            let table = Table {
                rows,
                shrink: vec![(1, 10), (0, 20)],
                keep_end: vec![],
            };
            for line in out.table(&table) {
                writeln!(w, "{line}")?;
            }
        }
    }
    errors_note(&resolved);
    Ok(ExitCode::SUCCESS)
}

fn walk_tree(
    n: &Resolved,
    prefix: &str,
    depth: usize,
    a: &TreeArgs,
    today: Date,
    out: Out,
    rows: &mut Vec<Row>,
) {
    if a.depth.is_some_and(|d| depth > d) {
        return;
    }
    let kids: Vec<&Resolved> = n
        .children
        .iter()
        .filter(|c| !(a.hide_done && matches!(c.status, Status::Done | Status::Dropped)))
        .collect();
    for (i, c) in kids.iter().enumerate() {
        let last = i + 1 == kids.len();
        let g = glyph(c);
        let settled = matches!(c.status, Status::Done | Status::Dropped | Status::Later);
        if out.plain {
            rows.push(Row {
                key: key(c),
                cells: vec![
                    vec![Cell::plain(depth.to_string())],
                    vec![Cell::plain(g.to_string())],
                    vec![Cell::plain(c.label.clone())],
                    vec![Cell::plain(c.status.as_str())],
                    vec![Cell::plain(c.progress.done.to_string())],
                    vec![Cell::plain(c.progress.total.to_string())],
                ],
            });
        } else {
            let mut facts = Vec::new();
            if let Some(gr) = &c.graft
                && !gr.path.is_empty()
            {
                let anchor = gr
                    .anchor
                    .as_ref()
                    .map(|a| format!("^{a}"))
                    .unwrap_or_default();
                facts.push(format!("{}{anchor}", gr.path));
            }
            if !c.tokens.people.is_empty() {
                facts.push(
                    c.tokens
                        .people
                        .iter()
                        .map(|p| format!("@{p}"))
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
            if c.tokens.priority > 0 {
                facts.push("!".repeat(usize::from(c.tokens.priority)));
            }
            if !settled && let Some(d) = c.due {
                facts.push(format!("due {d} ({})", relative(today.days_until(d))));
            }
            if !c.waits_on.is_empty() {
                facts.push(waits_text(&c.waits_on));
            }
            let style = if !c.children.is_empty() {
                Style::Bold
            } else if settled {
                Style::Dim
            } else {
                Style::Plain
            };
            rows.push(Row {
                key: key(c),
                cells: vec![
                    vec![
                        Cell::new(
                            format!("{prefix}{}", if last { "└─ " } else { "├─ " }),
                            Style::Dim,
                        ),
                        Cell::new(g.to_string(), Style::Mark(g)),
                        Cell::new(format!(" {}", c.label), style),
                    ],
                    vec![Cell::new(facts.join("  "), Style::Dim)],
                    vec![Cell::plain(progress(c))],
                ],
            });
        }
        let inner = format!("{prefix}{}", if last { "   " } else { "│  " });
        walk_tree(c, &inner, depth + 1, a, today, out, rows);
    }
}

// ----------------------------------------------------------------------------
// why

/// The node at `file:line` in that file's own tree, with its ancestors.
fn locate<'r>(
    r: &'r Resolution,
    file: &str,
    line: usize,
) -> Option<(Vec<&'r Resolved>, &'r Resolved)> {
    fn go<'r>(
        n: &'r Resolved,
        file: &str,
        line: usize,
        path: &mut Vec<&'r Resolved>,
    ) -> Option<&'r Resolved> {
        for c in n.children.iter().filter(|c| !c.grafted) {
            if c.file == file && c.line == line {
                return Some(c);
            }
            path.push(c);
            if let Some(hit) = go(c, file, line, path) {
                return Some(hit);
            }
            path.pop();
        }
        None
    }
    let tree = r.tree(file)?;
    let mut path = vec![&tree.root];
    let hit = go(&tree.root, file, line, &mut path)?;
    Some((path, hit))
}

/// What a node that waits on nothing is doing.
fn state(n: &Resolved) -> String {
    if !n.children.is_empty() && n.progress.total > 0 {
        format!(
            "{} of {} done, nothing blocks it",
            n.progress.done, n.progress.total
        )
    } else {
        match n.status {
            Status::Doing => "in progress, nothing blocks it".into(),
            Status::Question => "an open question".into(),
            Status::Done => "done".into(),
            _ => "ready to start".into(),
        }
    }
}

pub fn why(a: &WhyArgs, today: Date, out: Out) -> Result<ExitCode> {
    let mut groups = load::load(std::slice::from_ref(&a.plans))?;
    let g = groups.pop().expect("one path gives one workspace");
    let r = g.workspace.resolve(today);
    let item = match refs::resolve(&r, &a.item) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("sprig: {e}");
            return Ok(ExitCode::from(2));
        }
    };
    let (ancestors, node) =
        locate(&r, &item.file, item.line).ok_or("the item is not in its own file's tree")?;
    let loc = |n: &Resolved| g.display(&format!("{}:{}", n.file, n.line));
    let owner = |n: &Resolved| {
        n.people
            .first()
            .map(|p| format!("  @{p}"))
            .unwrap_or_default()
    };
    let mut rows: Vec<Row> = Vec::new();
    let head = |n: &Resolved, prefix: &str| -> Vec<Cell> {
        let g = glyph(n);
        vec![
            Cell::new(prefix.to_string(), Style::Dim),
            Cell::new(g.to_string(), Style::Mark(g)),
            Cell::new(format!(" {}", n.label), Style::Plain),
        ]
    };
    rows.push(Row {
        key: key(node),
        cells: vec![
            head(node, ""),
            vec![Cell::new(
                format!("{}{}", loc(node), owner(node)),
                Style::Dim,
            )],
        ],
    });
    if !node.blocked {
        let w = &mut io::stdout().lock();
        for line in out.table(&Table {
            rows,
            shrink: vec![(0, 20)],
            keep_end: vec![],
        }) {
            writeln!(w, "{line}")?;
        }
        writeln!(
            w,
            "{}",
            out.paint(&format!("not blocked: {}", state(node)), Style::Dim)
        )?;
        return Ok(ExitCode::SUCCESS);
    }
    // Blocking may come from an ancestor rather than the item's own waits.
    let from = if node.waits_on.is_empty() {
        let holder = ancestors
            .iter()
            .rev()
            .find(|a| !a.waits_on.is_empty())
            .copied()
            .unwrap_or(node);
        rows.push(Row {
            key: key(holder),
            cells: vec![
                vec![Cell::new(
                    format!("└─ inside {}, which waits:", holder.label),
                    Style::Dim,
                )],
                vec![Cell::new(loc(holder), Style::Dim)],
            ],
        });
        (holder, "   ".to_string())
    } else {
        (node, String::new())
    };
    let mut seen = HashSet::new();
    seen.insert((node.file.clone(), node.line));
    chain(&r, &g, from.0, &from.1, &mut seen, &mut rows);
    let mut w = io::stdout().lock();
    for line in out.table(&Table {
        rows,
        shrink: vec![(0, 20)],
        keep_end: vec![],
    }) {
        writeln!(w, "{line}")?;
    }
    Ok(ExitCode::from(1))
}

fn chain(
    r: &Resolution,
    g: &Loaded,
    n: &Resolved,
    prefix: &str,
    seen: &mut HashSet<(String, usize)>,
    rows: &mut Vec<Row>,
) {
    let loc = |n: &Resolved| g.display(&format!("{}:{}", n.file, n.line));
    for (i, w) in n.waits_on.iter().enumerate() {
        let last = i + 1 == n.waits_on.len();
        let branch = format!("{prefix}{}", if last { "└─ " } else { "├─ " });
        let inner = format!("{prefix}{}", if last { "   " } else { "│  " });
        let note = |text: String, style: Style| Row {
            key: String::new(),
            cells: vec![vec![
                Cell::new(format!("{inner}└─ "), Style::Dim),
                Cell::new(text, style),
            ]],
        };
        match w {
            Wait::Start(d) => rows.push(Row {
                key: String::new(),
                cells: vec![vec![
                    Cell::new(branch, Style::Dim),
                    Cell::plain(format!("starts {d}")),
                ]],
            }),
            Wait::Reference {
                label,
                unknown: true,
                ..
            } => rows.push(Row {
                key: String::new(),
                cells: vec![vec![
                    Cell::new(branch, Style::Dim),
                    Cell::new(format!("can't find {label}"), Style::Error),
                ]],
            }),
            Wait::Reference { target, .. } => {
                let Some((file, line)) = target else { continue };
                let Some((ancestors, t)) = locate(r, file, *line) else {
                    continue;
                };
                let g_ = glyph(t);
                let owner = t
                    .people
                    .first()
                    .map(|p| format!("  @{p}"))
                    .unwrap_or_default();
                let looped = !seen.insert((t.file.clone(), t.line));
                rows.push(Row {
                    key: key(t),
                    cells: vec![
                        vec![
                            Cell::new(branch, Style::Dim),
                            Cell::new(g_.to_string(), Style::Mark(g_)),
                            Cell::new(format!(" {}", t.label), Style::Plain),
                        ],
                        vec![Cell::new(format!("{}{owner}", loc(t)), Style::Dim)],
                    ],
                });
                if looped {
                    rows.push(note(
                        "a cycle: this was already on the chain".into(),
                        Style::Warning,
                    ));
                    continue;
                }
                if !t.waits_on.is_empty() {
                    chain(r, g, t, &inner, seen, rows);
                } else if t.blocked {
                    let holder = ancestors
                        .iter()
                        .rev()
                        .find(|a| !a.waits_on.is_empty())
                        .copied();
                    match holder {
                        Some(h) if seen.insert((h.file.clone(), h.line)) => {
                            rows.push(note(
                                format!("inside {}, which waits:", h.label),
                                Style::Dim,
                            ));
                            chain(r, g, h, &format!("{inner}   "), seen, rows);
                        }
                        Some(_) => rows.push(note(
                            "a cycle: this was already on the chain".into(),
                            Style::Warning,
                        )),
                        None => rows.push(note(state(t), Style::Plain)),
                    }
                } else {
                    rows.push(note(state(t), Style::Plain));
                }
            }
        }
    }
}

// ----------------------------------------------------------------------------
// status

pub fn status(a: &StatusArgs, today: Date, out: Out) -> Result<ExitCode> {
    let (groups, resolved) = resolve_all(&a.paths, today)?;
    let late = |e: &Entry| e.due_date.is_some_and(|d| d < today);

    if a.oneline {
        let trees: Vec<&Tree> = groups
            .iter()
            .zip(&resolved)
            .flat_map(|(g, r)| entry_trees(g, r))
            .collect();
        let n = next::next(&trees, None);
        let (done, total) = trees.iter().fold((0, 0), |(d, t), tr| {
            (d + tr.root.progress.done, t + tr.root.progress.total)
        });
        let pct = (done * 100).checked_div(total).unwrap_or(0);
        let late = n.ready.iter().chain(&n.waiting).filter(|e| late(e)).count();
        let mut line = format!(
            "sprig {pct}% · {} ready · {} waiting",
            n.ready.len(),
            n.waiting.len()
        );
        if late > 0 {
            line.push_str(&format!(" · {late} late"));
        }
        writeln!(io::stdout().lock(), "{}", clip(&line, 60))?;
        return Ok(if late > 0 {
            ExitCode::from(1)
        } else {
            ExitCode::SUCCESS
        });
    }

    let mut rows = Vec::new();
    for (g, r) in groups.iter().zip(&resolved) {
        for t in r.trees() {
            if g.entries.as_ref().is_some_and(|e| !e.contains(&t.file)) {
                continue;
            }
            let n = next::next(&[t], None);
            let late = n.ready.iter().chain(&n.waiting).filter(|e| late(e)).count();
            let errors = r
                .diagnostics
                .iter()
                .filter(|d| d.file == t.file && d.severity() == Severity::Error)
                .count();
            let p = t.root.progress;
            let shown = g.display(&t.file);
            if out.plain {
                rows.push(Row {
                    key: shown,
                    cells: [
                        p.done,
                        p.total,
                        n.ready.len(),
                        n.waiting.len(),
                        late,
                        errors,
                    ]
                    .iter()
                    .map(|v| vec![Cell::plain(v.to_string())])
                    .collect(),
                });
                continue;
            }
            const BAR: usize = 10;
            let filled = (p.done * BAR).checked_div(p.total).unwrap_or(0);
            let count = |n: usize, what: &str, style: Style| {
                Cell::new(
                    if n > 0 {
                        format!("{n} {what}")
                    } else {
                        String::new()
                    },
                    style,
                )
            };
            rows.push(Row {
                key: shown.clone(),
                cells: vec![
                    vec![Cell::plain(shown.trim_end_matches(".sprig").to_string())],
                    vec![
                        Cell::new("█".repeat(filled), Style::Mark('x')),
                        Cell::new("░".repeat(BAR - filled), Style::Dim),
                    ],
                    vec![Cell::plain(format!("{}/{}", p.done, p.total))],
                    vec![count(n.ready.len(), "ready", Style::Plain)],
                    vec![count(n.waiting.len(), "waiting", Style::Dim)],
                    vec![count(late, "late", Style::Error)],
                    vec![count(
                        errors,
                        if errors == 1 { "error" } else { "errors" },
                        Style::Error,
                    )],
                ],
            });
        }
    }
    let mut w = io::stdout().lock();
    for line in out.table(&Table {
        rows,
        shrink: vec![(0, 8)],
        keep_end: vec![0],
    }) {
        writeln!(w, "{line}")?;
    }
    Ok(ExitCode::SUCCESS)
}

// ----------------------------------------------------------------------------
// completion candidates

/// Anchors as refs, or people, from the plans in the current folder, one per
/// line, for the shell scripts `sprig completions` prints.
pub fn candidates(what: &str, today: Date) -> Result<ExitCode> {
    let (_, resolved) = resolve_all(&[std::path::PathBuf::from(".")], today)?;
    let mut found = std::collections::BTreeSet::new();
    for r in &resolved {
        for p in r.parsed.values() {
            sprig_core::syntax::walk(&p.document.root, &mut |n: &sprig_core::Node| match what {
                "refs" => {
                    if let Some(a) = n.tokens.anchor() {
                        found.insert(format!("^{}", a.name));
                    }
                }
                "people" => {
                    found.extend(n.tokens.people.iter().cloned());
                }
                _ => {}
            });
            if what == "people" {
                found.extend(p.document.root.tokens.people.iter().cloned());
            }
        }
    }
    let mut w = io::stdout().lock();
    for f in found {
        writeln!(w, "{f}")?;
    }
    Ok(ExitCode::SUCCESS)
}
