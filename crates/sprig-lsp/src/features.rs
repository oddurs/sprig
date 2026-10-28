//! What the server answers: each a function of a snapshot of the workspace,
//! one document in it and a position, so it can be tested without a client.

use std::collections::BTreeSet;

use lsp_types::{self as lsp, Position, Range, Uri};
use sprig_core::edit::{self, Change};
use sprig_core::syntax::{self, LineKind, Node};
use sprig_core::token::{self, Link, Tokens};
use sprig_core::workspace::link_target;
use sprig_core::{Date, Mark, Resolved, Span, Status, Wait};

use crate::Snapshot;
use crate::pos::{Encoding, Lines};

pub fn diagnostic(d: &sprig_core::Diagnostic, lines: &Lines, enc: Encoding) -> lsp::Diagnostic {
    let mut message = d.message.clone();
    if let Some(help) = &d.help {
        message.push('\n');
        message.push_str(help);
    }
    lsp::Diagnostic {
        range: lines.range(d.span, enc),
        severity: Some(match d.severity() {
            sprig_core::Severity::Error => lsp::DiagnosticSeverity::ERROR,
            sprig_core::Severity::Warning => lsp::DiagnosticSeverity::WARNING,
        }),
        code: Some(lsp::NumberOrString::String(d.code.to_string())),
        source: Some("sprig".into()),
        message,
        ..Default::default()
    }
}

// ----------------------------------------------------------------------------
// Finding what is under the cursor

/// The tokens of every line that has them, with the line: the title, items
/// and answers.
fn token_lines(root: &Node) -> Vec<(usize, &Tokens)> {
    let mut out = Vec::new();
    if root.span.is_some() {
        out.push((root.line, &root.tokens));
    }
    for a in &root.answers {
        out.push((a.line, &a.tokens));
    }
    syntax::walk(root, &mut |n: &Node| {
        out.push((n.line, &n.tokens));
        for a in &n.answers {
            out.push((a.line, &a.tokens));
        }
    });
    out
}

fn tokens_on(root: &Node, line: usize) -> Option<&Tokens> {
    token_lines(root)
        .into_iter()
        .find(|(l, _)| *l == line)
        .map(|(_, t)| t)
}

fn item_on(root: &Node, line: usize) -> Option<&Node> {
    let mut found = None;
    syntax::walk(root, &mut |n: &Node| {
        if n.line == line {
            found = Some(n);
        }
    });
    found
}

/// What a reference or link names.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Target {
    Anchor { file: String, anchor: String },
    File(String),
}

fn link_names(from: &str, link: &Link) -> Option<Target> {
    let file = link_target(from, &link.path)?;
    Some(match &link.anchor {
        Some(a) => Target::Anchor {
            file,
            anchor: a.clone(),
        },
        None => Target::File(file),
    })
}

/// What an `after:` value names (§7.2).
fn after_names(from: &str, value: &str) -> Option<Target> {
    if let Some(anchor) = value.strip_prefix('^') {
        return Some(Target::Anchor {
            file: from.to_string(),
            anchor: anchor.to_string(),
        });
    }
    let (_, link) = token::links(value).into_iter().next()?;
    link_names(from, &link)
}

fn inside(span: Span, offset: usize) -> bool {
    span.start <= offset && offset <= span.end
}

/// The reference, link or anchor under the cursor, and what it names. An
/// anchor names itself.
fn target_at(snap: &Snapshot, key: &str, offset: usize, line: usize) -> Option<Target> {
    let parsed = snap.res.parsed.get(key)?;
    let tokens = tokens_on(&parsed.document.root, line)?;
    if let Some(a) = tokens.after.iter().find(|a| inside(a.span, offset)) {
        return after_names(key, &a.value);
    }
    if let Some(l) = tokens.links.iter().find(|l| inside(l.span, offset)) {
        return link_names(key, l);
    }
    if let Some(a) = tokens.anchors.iter().find(|a| inside(a.span, offset)) {
        return Some(Target::Anchor {
            file: key.to_string(),
            anchor: a.name.clone(),
        });
    }
    None
}

/// Where a target is written: the item with the anchor, its first if two
/// share it (Appendix B.4), or the title of a file.
fn place_of(snap: &Snapshot, target: &Target) -> Option<(String, Span)> {
    match target {
        Target::Anchor { file, anchor } => {
            let root = &snap.res.parsed.get(file)?.document.root;
            let mut found = None;
            syntax::walk(root, &mut |n: &Node| {
                if found.is_none() && n.tokens.anchor().is_some_and(|a| &a.name == anchor) {
                    found = n.span;
                }
            });
            Some((file.clone(), found?))
        }
        Target::File(file) => {
            let root = &snap.res.parsed.get(file)?.document.root;
            Some((file.clone(), root.span.unwrap_or(Span::new(0, 0))))
        }
    }
}

fn range_in(snap: &Snapshot, file: &str, span: Span, enc: Encoding) -> Option<Range> {
    Some(Lines::new(snap.files.get(file)?).range(span, enc))
}

// ----------------------------------------------------------------------------
// Definition and references (cairn 0051)

pub fn definition(
    snap: &Snapshot,
    key: &str,
    enc: Encoding,
    pos: Position,
) -> Vec<(String, Range)> {
    let Some(src) = snap.files.get(key) else {
        return Vec::new();
    };
    let lines = Lines::new(src);
    let offset = lines.offset(pos, enc);
    let Some(target) = target_at(snap, key, offset, pos.line as usize + 1) else {
        return Vec::new();
    };
    place_of(snap, &target)
        .and_then(|(file, span)| Some((file.clone(), range_in(snap, &file, span, enc)?)))
        .into_iter()
        .collect()
}

/// Every `after:` and link in the workspace that names what is under the
/// cursor, and with `declaration`, the anchor that defines it.
pub fn references(
    snap: &Snapshot,
    key: &str,
    enc: Encoding,
    pos: Position,
    declaration: bool,
) -> Vec<(String, Range)> {
    let Some(src) = snap.files.get(key) else {
        return Vec::new();
    };
    let offset = Lines::new(src).offset(pos, enc);
    let Some(target) = target_at(snap, key, offset, pos.line as usize + 1) else {
        return Vec::new();
    };
    let mut out: Vec<(String, Span)> = Vec::new();
    for (file, parsed) in &snap.res.parsed {
        for (_, tokens) in token_lines(&parsed.document.root) {
            for a in &tokens.after {
                if after_names(file, &a.value).as_ref() == Some(&target) {
                    out.push((file.clone(), a.span));
                }
            }
            for l in &tokens.links {
                if link_names(file, l).as_ref() == Some(&target) {
                    out.push((file.clone(), l.span));
                }
            }
            if declaration
                && let Target::Anchor { file: f, anchor } = &target
                && f == file
                && let Some(a) = tokens.anchor().filter(|a| &a.name == anchor)
            {
                out.push((file.clone(), a.span));
            }
        }
    }
    out.sort_by_key(|(f, s)| (f.clone(), s.start));
    out.into_iter()
        .filter_map(|(f, s)| Some((f.clone(), range_in(snap, &f, s, enc)?)))
        .collect()
}

// ----------------------------------------------------------------------------
// Completion (cairn 0050)

const FIELDS: [(&str, &str); 5] = [
    (
        "due",
        "When it should be finished: a date, or a shortcut such as fri or +2w",
    ),
    (
        "start",
        "The first day work can begin; it is blocked until then",
    ),
    ("est", "Estimated work: 4h, 2d (8 hours) or 1w (40 hours)"),
    (
        "every",
        "Recurring upkeep: day, week, month, 3d, 2w or a weekday",
    ),
    (
        "after",
        "What it waits on: ^anchor, [[file^anchor]] or [[file]]",
    ),
];

pub fn completion(
    snap: &Snapshot,
    key: &str,
    enc: Encoding,
    pos: Position,
) -> Vec<lsp::CompletionItem> {
    let Some(src) = snap.files.get(key) else {
        return Vec::new();
    };
    let lines = Lines::new(src);
    let offset = lines.offset(pos, enc);
    let line = lines.line_of(offset);
    // Notes and comments carry no tokens (§2.6), so nothing is offered there.
    let kind = syntax::lex(src).get(line - 1).map(|l| l.kind);
    if !matches!(
        kind,
        Some(LineKind::Item(_) | LineKind::Answer | LineKind::Title)
    ) {
        return Vec::new();
    }
    let line_start = lines.line_span(line).start;
    let before = &src[line_start..offset];
    let word_at = before
        .rfind(char::is_whitespace)
        .map_or(line_start, |i| line_start + i + 1);
    let word = &src[word_at..offset];
    let after_cursor = &src[offset..lines.line_span(line).end];
    let close = if after_cursor.starts_with("]]") {
        ""
    } else {
        "]]"
    };
    let item = |label: String, from: usize, text: String, kind, detail: Option<String>| {
        lsp::CompletionItem {
            label,
            kind: Some(kind),
            detail,
            filter_text: Some(text.clone()),
            text_edit: Some(lsp::CompletionTextEdit::Edit(lsp::TextEdit {
                range: lines.range(Span::new(from, offset), enc),
                new_text: text,
            })),
            ..Default::default()
        }
    };

    // A link being written: `[[kit` offers files, `[[lease^` that file's anchors.
    if let Some(open) = word.rfind("[[")
        && !word[open..].contains("]]")
    {
        let inner_at = word_at + open + 2;
        let inner = &src[inner_at..offset];
        if let Some(caret) = inner.find('^') {
            let link = format!("{}.sprig", inner[..caret].trim_end_matches(".sprig"));
            let Some(file) = link_target(key, link.trim_start_matches("./")) else {
                return Vec::new();
            };
            return anchors(snap, &file)
                .into_iter()
                .map(|(name, text, _)| {
                    let mut i = item(
                        name.clone(),
                        inner_at + caret + 1,
                        format!("{name}{close}"),
                        lsp::CompletionItemKind::REFERENCE,
                        Some(text),
                    );
                    i.filter_text = Some(name);
                    i
                })
                .collect();
        }
        return snap
            .files
            .keys()
            .filter(|f| f.as_str() != key)
            .map(|f| {
                let rel = relative(key, f);
                let title = snap
                    .res
                    .parsed
                    .get(f)
                    .and_then(|p| p.document.title.clone());
                let mut i = item(
                    rel.clone(),
                    inner_at,
                    format!("{rel}{close}"),
                    lsp::CompletionItemKind::FILE,
                    title,
                );
                i.filter_text = Some(rel);
                i
            })
            .collect();
    }

    // An anchor, in `after:^` or on its own.
    if let Some(name) = word
        .strip_prefix("after:")
        .unwrap_or(word)
        .strip_prefix('^')
    {
        let from = offset - name.len();
        // Not the anchor being typed, which is already in the text.
        return anchors(snap, key)
            .into_iter()
            .filter(|(_, _, l)| *l != line)
            .map(|(name, text, _)| {
                let mut i = item(
                    name.clone(),
                    from,
                    name.clone(),
                    lsp::CompletionItemKind::REFERENCE,
                    Some(text),
                );
                i.filter_text = Some(name);
                i
            })
            .collect();
    }

    if word.starts_with('@') {
        return people(snap)
            .into_iter()
            .map(|p| {
                let label = format!("@{p}");
                item(
                    label.clone(),
                    word_at,
                    label,
                    lsp::CompletionItemKind::VALUE,
                    None,
                )
            })
            .collect();
    }

    // A field key, once its first letters are typed, so that ordinary words
    // are not interrupted.
    if !word.is_empty() && word.chars().all(|c| c.is_ascii_alphabetic()) {
        let lower = word.to_ascii_lowercase();
        return FIELDS
            .iter()
            .filter(|(k, _)| k.starts_with(&lower))
            .map(|(k, doc)| {
                let label = format!("{k}:");
                item(
                    label.clone(),
                    word_at,
                    label,
                    lsp::CompletionItemKind::PROPERTY,
                    Some(doc.to_string()),
                )
            })
            .collect();
    }
    Vec::new()
}

/// A file's anchors with their items' text and line, first use only
/// (Appendix B.4).
fn anchors(snap: &Snapshot, file: &str) -> Vec<(String, String, usize)> {
    let Some(parsed) = snap.res.parsed.get(file) else {
        return Vec::new();
    };
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    syntax::walk(&parsed.document.root, &mut |n: &Node| {
        if let Some(a) = n.tokens.anchor()
            && seen.insert(a.name.clone())
        {
            out.push((a.name.clone(), n.tokens.text.clone(), n.line));
        }
    });
    out
}

/// Everyone named anywhere in the workspace, in order.
fn people(snap: &Snapshot) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for parsed in snap.res.parsed.values() {
        for (_, t) in token_lines(&parsed.document.root) {
            out.extend(t.people.iter().cloned());
        }
    }
    out
}

/// How a link written in `from` names `to`: relative to `from`'s directory,
/// without `.sprig`.
fn relative(from: &str, to: &str) -> String {
    let mut dir: Vec<&str> = from.split('/').collect();
    dir.pop();
    let to: Vec<&str> = to.split('/').collect();
    let (to_dir, name) = to.split_at(to.len() - 1);
    let common = dir.iter().zip(to_dir).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<&str> = vec![".."; dir.len() - common];
    parts.extend(&to_dir[common..]);
    parts.push(name[0].strip_suffix(".sprig").unwrap_or(name[0]));
    parts.join("/")
}

// ----------------------------------------------------------------------------
// Hover (cairn 0052)

pub fn hover(snap: &Snapshot, key: &str, enc: Encoding, pos: Position) -> Option<lsp::Hover> {
    let src = snap.files.get(key)?;
    let lines = Lines::new(src);
    let line = pos.line as usize + 1;
    let tree = snap.res.tree(key)?;
    let node = find_resolved(&tree.root, key, line)?;
    let span = snap.res.parsed.get(key).and_then(|p| {
        let root = &p.document.root;
        if root.line == line {
            root.span
        } else {
            item_on(root, line).and_then(|n| n.span)
        }
    })?;
    Some(lsp::Hover {
        contents: lsp::HoverContents::Markup(lsp::MarkupContent {
            kind: lsp::MarkupKind::Markdown,
            value: describe(node, snap.today),
        }),
        range: Some(lines.range(span, enc)),
    })
}

/// The node a file writes on `line`, in its own tree: not one a graft
/// brought in.
fn find_resolved<'a>(n: &'a Resolved, file: &str, line: usize) -> Option<&'a Resolved> {
    if n.grafted {
        return None;
    }
    if n.file == file && n.line == line {
        return Some(n);
    }
    n.children.iter().find_map(|c| find_resolved(c, file, line))
}

fn describe(n: &Resolved, today: Date) -> String {
    let mut out = Vec::new();
    let p = n.progress;
    let mut head = format!("**{}**", status_word(n.status));
    if p.total > 0 && !(n.is_leaf() && n.graft.is_none()) {
        head += &format!(" · {} of {} done", p.done, p.total);
        if p.doing > 0 {
            head += &format!(", {} doing", p.doing);
        }
    }
    if n.estimate > 0.0 {
        head += &format!(" · {} left", hours(n.estimate));
    }
    if !n.people.is_empty() {
        let who: Vec<String> = n.people.iter().map(|p| format!("@{p}")).collect();
        head += &format!(" · {}", who.join(" "));
    }
    out.push(head);
    if let Some(g) = &n.graft
        && g.mounted
    {
        let what = match &g.anchor {
            Some(a) => format!("`{}^{a}`", g.path),
            None => format!("`{}`", g.path),
        };
        out.push(format!("Mounts {what}, *{}*.", g.title));
    }
    let mut dates = Vec::new();
    if let Some(d) = n.due {
        dates.push(format!("Due {}", when(d, today)));
    }
    if let Some(d) = n.start {
        dates.push(format!("starts {}", when(d, today)));
    }
    if !dates.is_empty() {
        out.push(dates.join(", ") + ".");
    }
    if n.blocked {
        if n.waits_on.is_empty() {
            out.push("Blocked: something above it is waiting.".into());
        } else {
            let mut waits = vec!["Waiting on:".to_string()];
            for w in &n.waits_on {
                waits.push(match w {
                    Wait::Reference {
                        label,
                        unknown: true,
                        ..
                    } => format!("- `{label}`, which names nothing"),
                    Wait::Reference {
                        label,
                        target: Some((file, line)),
                        ..
                    } => format!("- *{label}* ({file}:{line})"),
                    Wait::Reference { label, .. } => format!("- *{label}*"),
                    Wait::Start(d) => format!("- its start date, {}", when(*d, today)),
                });
            }
            out.push(waits.join("\n"));
        }
    }
    if n.ready_to_close {
        out.push("Everything beneath it is done: ready to close.".into());
    }
    out.join("\n\n")
}

fn status_word(s: Status) -> &'static str {
    match s {
        Status::Todo => "To do",
        Status::Doing => "Doing",
        Status::Done => "Done",
        Status::Dropped => "Dropped",
        Status::Later => "Later",
        Status::Question => "Question",
        Status::Empty => "Nothing to do yet",
    }
}

fn when(d: Date, today: Date) -> String {
    let days = today.days_until(d);
    let rel = match days {
        0 => "today".to_string(),
        1 => "tomorrow".to_string(),
        -1 => "yesterday".to_string(),
        d if d > 0 => format!("in {d} days"),
        d => format!("{} days ago", -d),
    };
    format!("{d} ({rel})")
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

// ----------------------------------------------------------------------------
// Code actions and formatting (cairn 0053)

pub fn actions(
    snap: &Snapshot,
    key: &str,
    uri: &Uri,
    enc: Encoding,
    range: Range,
) -> Vec<lsp::CodeActionOrCommand> {
    let Some(src) = snap.files.get(key) else {
        return Vec::new();
    };
    let lines = Lines::new(src);
    let line = range.start.line as usize + 1;
    let Some(item) = snap
        .res
        .parsed
        .get(key)
        .and_then(|p| item_on(&p.document.root, line))
    else {
        return Vec::new();
    };
    let mark = item.mark.expect("an item has a mark");
    let mut out = Vec::new();
    let mut action = |title: String, change: Change, preferred: bool| {
        if change.is_empty() {
            return;
        }
        let edits = text_edits(&lines, &change, enc);
        out.push(lsp::CodeActionOrCommand::CodeAction(lsp::CodeAction {
            title,
            kind: Some(lsp::CodeActionKind::REFACTOR_REWRITE),
            edit: Some(lsp::WorkspaceEdit {
                changes: Some([(uri.clone(), edits)].into_iter().collect()),
                ..Default::default()
            }),
            is_preferred: Some(preferred),
            ..Default::default()
        }));
    };
    if !mark.is_structure() {
        if let Ok((change, moved)) = edit::tick(src, line, snap.today) {
            let title = match (moved, mark) {
                (Some(d), _) => format!("Done for now: move due to {d}"),
                (None, Mark::Done) => "Reopen".to_string(),
                (None, _) => "Mark done".to_string(),
            };
            action(title, change, true);
        }
        // The working cycle: to do, doing, done, and round again.
        let next = match mark {
            Mark::Todo => Some((Mark::Doing, "Start: mark doing (~)")),
            Mark::Doing => Some((Mark::Done, "Finish: mark done (x)")),
            Mark::Done => Some((Mark::Todo, "Cycle back to to do (-)")),
            _ => None,
        };
        if let Some((m, title)) = next
            && let Ok(change) = edit::set_mark(src, line, m)
            && !(m == Mark::Done && item.tokens.field("every").is_some())
        {
            action(title.to_string(), change, false);
        }
    }
    // Typed dates written out, value only, as a formatter would (§9.4).
    for f in &item.tokens.fields {
        if !matches!(f.key.as_str(), "due" | "start") || Date::parse_iso(&f.value).is_some() {
            continue;
        }
        let Some(d) = sprig_core::date::resolve(&f.value, snap.today) else {
            continue;
        };
        let written = &src[f.span.start..f.span.end];
        let colon = written.find(':').expect("a field has a colon");
        let change = Change {
            source: String::new(),
            edits: vec![edit::Replace {
                span: Span::new(f.span.start + colon + 1, f.span.end),
                text: d.to_string(),
            }],
        };
        action(format!("Write {}:{} as {d}", f.key, f.value), change, false);
    }
    out
}

pub fn format(snap: &Snapshot, key: &str, enc: Encoding) -> Vec<lsp::TextEdit> {
    let Some(src) = snap.files.get(key) else {
        return Vec::new();
    };
    text_edits(&Lines::new(src), &edit::format(src, snap.today), enc)
}

fn text_edits(lines: &Lines, change: &Change, enc: Encoding) -> Vec<lsp::TextEdit> {
    change
        .edits
        .iter()
        .map(|e| lsp::TextEdit {
            range: lines.range(e.span, enc),
            new_text: e.text.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_written_relative_to_the_linking_file() {
        assert_eq!(relative("bakery.sprig", "lease.sprig"), "lease");
        assert_eq!(relative("bakery.sprig", "rooms/hall.sprig"), "rooms/hall");
        assert_eq!(relative("rooms/kitchen.sprig", "rooms/hall.sprig"), "hall");
        assert_eq!(relative("rooms/kitchen.sprig", "lease.sprig"), "../lease");
        assert_eq!(relative("a/b/c.sprig", "a/d/e.sprig"), "../d/e");
    }
}
