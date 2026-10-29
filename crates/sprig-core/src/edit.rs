//! Edits that change only the bytes they mean to.
//!
//! Sprig is line-oriented: every byte of a file belongs to exactly one line,
//! and every line is an item, a note, an answer, a comment or blank. So the
//! source text itself is the lossless layer that format-preserving editors
//! (toml_edit's decor, rowan's trivia) build a tree to keep: an edit is a set
//! of replacements of whole or partial lines, located by the parser's spans,
//! and whatever an edit does not name is copied through untouched. Indentation
//! style, line endings, a BOM, comments and blank lines all survive by
//! construction. (cairn 0042 records the argument.)
//!
//! Every operation returns a [`Change`]: the new source, and the replacements
//! that produce it from the old one, in the old one's byte offsets, so an
//! editor can send exactly those as text edits.
//!
//! # Example
//!
//! ```
//! use sprig_core::edit::{self, Place, Replace};
//! use sprig_core::{Date, Mark, Span};
//!
//! let today = Date::new(2026, 10, 1).unwrap();
//! let src = "Plan\n\n- Logo\n    - Sketch\n";
//!
//! // Each edit returns the new text and the replacements that made it, and
//! // leaves every other byte alone.
//! let (c, _) = edit::tick(src, 3, today).unwrap();
//! assert_eq!(c.source, "Plan\n\nx Logo\n    - Sketch\n");
//! let c = edit::set_mark(src, 4, Mark::Doing).unwrap();
//! assert_eq!(c.edits.len(), 1);
//! let c = edit::set_field(src, 3, "due", Some("2026-10-09")).unwrap();
//! assert!(c.source.contains("- Logo  due:2026-10-09"));
//! // A new child takes its siblings' indentation.
//! let c = edit::add_item(src, 3, Place::Child, "- Colours").unwrap();
//! assert!(c.source.ends_with("    - Sketch\n    - Colours\n"));
//! let c = edit::add_answer("Q\n\n? Which?\n", 3, "This one").unwrap();
//! assert!(c.source.ends_with("  = This one\n"));
//!
//! // Formatting writes two spaces a level and writes typed dates out.
//! let c = edit::format("Plan\n\n- Logo  due:fri\n    - Sketch\n", today);
//! assert_eq!(c.source, "Plan\n\n- Logo  due:2026-10-02\n  - Sketch\n");
//! assert!(edit::format(&c.source, today).is_empty());
//!
//! let same = edit::apply("abc", &[Replace { span: Span::new(1, 2), text: "B".into() }]);
//! assert_eq!(same, "aBc");
//! ```

use std::fmt;

use crate::date::Date;
use crate::diagnostic::Span;
use crate::syntax::{self, Mark, Node};
use crate::token;

/// One replacement, in the original source's bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Replace {
    /// The bytes of the original source it replaces.
    pub span: Span,
    /// What goes there instead.
    pub text: String,
}

/// The result of an edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    /// The whole file after the change.
    pub source: String,
    /// Sorted, and never overlapping. Empty when nothing changed.
    pub edits: Vec<Replace>,
}

impl Change {
    fn new(src: &str, mut edits: Vec<Replace>) -> Change {
        edits.retain(|e| src[e.span.start..e.span.end] != e.text);
        edits.sort_by_key(|e| (e.span.start, e.span.end));
        Change {
            source: apply(src, &edits),
            edits,
        }
    }

    /// Whether the change leaves the file as it was.
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }
}

/// Applies sorted, non-overlapping replacements.
pub fn apply(src: &str, edits: &[Replace]) -> String {
    let mut out = String::with_capacity(src.len());
    let mut at = 0;
    for e in edits {
        out.push_str(&src[at..e.span.start]);
        out.push_str(&e.text);
        at = e.span.end;
    }
    out.push_str(&src[at..]);
    out
}

/// Why an edit could not be made.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum EditError {
    /// No item starts on that line.
    NotAnItem(usize),
    /// The item is a group or a graft, which have no status of their own.
    NoStatus(usize),
    /// The text to add is not one line, or does not start with a mark.
    BadText(String),
    /// The edit would not read back as written, such as a quoted value on a
    /// line with an unclosed quote, which would pair with it (§4.1).
    WouldNotRead(usize),
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditError::NotAnItem(l) => write!(f, "line {l} is not an item"),
            EditError::NoStatus(l) => write!(
                f,
                "line {l} is a group or a graft, whose status comes from its children"
            ),
            EditError::BadText(t) => write!(f, "{t:?} is not one item line"),
            EditError::WouldNotRead(l) => write!(
                f,
                "line {l} has an unclosed quote, so the field would not read back; close it first"
            ),
        }
    }
}

impl std::error::Error for EditError {}

// ----------------------------------------------------------------------------
// Lines

/// One line of a file, by byte offsets.
#[derive(Clone, Copy, Debug)]
struct Raw {
    start: usize,
    /// Excludes the line ending.
    end: usize,
    /// After the line ending, or `end` on the last line without one.
    next: usize,
    /// Where the leading spaces and tabs stop.
    text: usize,
    /// Indentation in columns, a tab counting two (§3.1).
    indent: usize,
}

fn raw_lines(src: &str) -> Vec<Raw> {
    let mut out = Vec::new();
    let mut at = if src.starts_with('\u{feff}') { 3 } else { 0 };
    loop {
        let rest = &src[at..];
        let nl = rest.find('\n');
        let line_end = nl.map_or(src.len(), |i| at + i);
        let end = if src[at..line_end].ends_with('\r') {
            line_end - 1
        } else {
            line_end
        };
        let mut indent = 0;
        let mut text = at;
        for b in src[at..end].bytes() {
            match b {
                b' ' => indent += 1,
                b'\t' => indent += 2,
                _ => break,
            }
            text += 1;
        }
        let next = nl.map_or(src.len(), |i| at + i + 1);
        out.push(Raw {
            start: at,
            end,
            next,
            text,
            indent,
        });
        match nl {
            Some(_) => at = next,
            None => break,
        }
    }
    out
}

/// The file's line ending, by its first line: CRLF if that is what it uses.
fn eol(src: &str) -> &'static str {
    match src.find('\n') {
        Some(i) if i > 0 && src.as_bytes()[i - 1] == b'\r' => "\r\n",
        _ => "\n",
    }
}

fn find_item(root: &Node, line: usize) -> Option<&Node> {
    let mut hit = None;
    syntax::walk(root, &mut |n: &Node| {
        if n.line == line {
            hit = Some(n);
        }
    });
    hit
}

/// The last line, 1-based, of an item's branch: its own line and every line
/// after it that is blank, a comment, or indented deeper, up to the last one
/// that is not blank.
fn branch_end(lines: &[Raw], src: &str, line: usize) -> usize {
    let indent = lines[line - 1].indent;
    let mut last = line;
    for (i, l) in lines.iter().enumerate().skip(line) {
        let text = &src[l.text..l.end];
        if text.trim().is_empty() {
            continue;
        }
        if l.indent <= indent && !text.starts_with("//") {
            break;
        }
        last = i + 1;
    }
    last
}

// ----------------------------------------------------------------------------
// Operations

/// Rewrites the mark of the item on `line` (1-based).
pub fn set_mark(src: &str, line: usize, mark: Mark) -> Result<Change, EditError> {
    let doc = syntax::parse("", src).document;
    let item = find_item(&doc.root, line).ok_or(EditError::NotAnItem(line))?;
    let current = item.mark.expect("an item has a mark");
    if current.is_structure() || mark.is_structure() {
        return Err(EditError::NoStatus(line));
    }
    let at = item.span.expect("an item has a span").start;
    Ok(Change::new(
        src,
        vec![Replace {
            span: Span::new(at, at + 1),
            text: mark.as_char().to_string(),
        }],
    ))
}

/// Ticks the item on `line` as [`syntax::tick`] does: done reopens, a
/// recurring item moves its due date, anything else becomes done.
pub fn tick(src: &str, line: usize, today: Date) -> Result<(Change, Option<Date>), EditError> {
    let doc = syntax::parse("", src).document;
    let item = find_item(&doc.root, line).ok_or(EditError::NotAnItem(line))?;
    if item.mark.is_some_and(Mark::is_structure) {
        return Err(EditError::NoStatus(line));
    }
    let raw = raw_lines(src)[line - 1];
    let text = &src[raw.start..raw.end];
    let ticked = syntax::tick(text, today).ok_or(EditError::NoStatus(line))?;
    let edits = line_diff(raw.start, text, &ticked.line);
    Ok((Change::new(src, edits), ticked.moved_to))
}

/// Sets `key` on the item on `line` to `value`, or removes it with `None`.
/// An existing field is changed where it is written; a new one is appended to
/// the line after two spaces. A value with whitespace is quoted.
pub fn set_field(
    src: &str,
    line: usize,
    key: &str,
    value: Option<&str>,
) -> Result<Change, EditError> {
    let doc = syntax::parse("", src).document;
    let item = find_item(&doc.root, line).ok_or(EditError::NotAnItem(line))?;
    let key = key.to_ascii_lowercase();
    let written: Vec<&token::Field> = item.tokens.fields.iter().filter(|f| f.key == key).collect();
    let token = value.map(|v| {
        if v.contains(char::is_whitespace) {
            format!("{key}:\"{v}\"")
        } else {
            format!("{key}:{v}")
        }
    });
    let mut edits = Vec::new();
    match (written.last(), token) {
        (Some(last), Some(t)) => edits.push(Replace {
            span: last.span,
            text: t,
        }),
        (None, Some(t)) => {
            let end = item.span.expect("an item has a span").end;
            edits.push(Replace {
                span: Span::new(end, end),
                text: format!("  {t}"),
            });
        }
        (_, None) => {}
    }
    if value.is_none() {
        // Each occurrence goes with the whitespace before it, so the line
        // closes up the way it was written.
        for f in written {
            let before = src[..f.span.start].trim_end_matches([' ', '\t']).len();
            edits.push(Replace {
                span: Span::new(before, f.span.end),
                text: String::new(),
            });
        }
    }
    let change = Change::new(src, edits);
    // Refuse rather than write a line that means something else.
    let reread = syntax::parse("", &change.source).document;
    let reads_back = find_item(&reread.root, line).is_some_and(|n| {
        if key == "after" {
            // after: values are collected, not overwritten (§4.7).
            value.is_none_or(|v| n.tokens.after.last().is_some_and(|a| a.value == v))
        } else {
            n.tokens.field(&key).map(|f| f.value.as_str()) == value
        }
    });
    if !reads_back {
        return Err(EditError::WouldNotRead(line));
    }
    Ok(change)
}

/// Where an added line goes, relative to an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    /// Its last child, one level in.
    Child,
    /// Right after its branch, at its level.
    Sibling,
}

/// Adds an item line (`"- Buy milk  @jo"`) under or after the item on `line`.
/// A child takes the indentation of the last child already there, or two
/// spaces more than its parent.
pub fn add_item(src: &str, line: usize, place: Place, text: &str) -> Result<Change, EditError> {
    let text = text.trim();
    let one_item = !text.contains(['\n', '\r'])
        && syntax::lex(&format!("_\n{text}"))
            .get(1)
            .is_some_and(|l| matches!(l.kind, syntax::LineKind::Item(_)));
    if !one_item {
        return Err(EditError::BadText(text.to_string()));
    }
    let doc = syntax::parse("", src).document;
    let item = find_item(&doc.root, line).ok_or(EditError::NotAnItem(line))?;
    let lines = raw_lines(src);
    let indent_of = |l: usize| {
        let r = lines[l - 1];
        src[r.start..r.text].to_string()
    };
    let indent = match place {
        // The last child's, since the new one goes right after its branch:
        // anything deeper would make it that child's child.
        Place::Child => match item.children.last() {
            Some(c) => indent_of(c.line),
            None => format!("{}  ", indent_of(line)),
        },
        Place::Sibling => indent_of(line),
    };
    insert_after(
        src,
        &lines,
        branch_end(&lines, src, line),
        &format!("{indent}{text}"),
    )
}

/// Adds an answer (`= text`) to the item on `line`, after its own notes and
/// answers and before its children.
pub fn add_answer(src: &str, line: usize, text: &str) -> Result<Change, EditError> {
    let text = text.trim();
    if text.contains(['\n', '\r']) {
        return Err(EditError::BadText(text.to_string()));
    }
    let doc = syntax::parse("", src).document;
    let item = find_item(&doc.root, line).ok_or(EditError::NotAnItem(line))?;
    let lines = raw_lines(src);
    let indent = match item.children.first() {
        Some(c) => {
            let r = lines[c.line - 1];
            src[r.start..r.text].to_string()
        }
        None => {
            let r = lines[line - 1];
            format!("{}  ", &src[r.start..r.text])
        }
    };
    // After the item's own lines: up to its first child, or its whole branch.
    let last = match item.children.first() {
        Some(c) => {
            let mut l = c.line - 1;
            while l > line && src[lines[l - 1].start..lines[l - 1].end].trim().is_empty() {
                l -= 1;
            }
            l
        }
        None => branch_end(&lines, src, line),
    };
    let answer = if text.is_empty() {
        "=".to_string()
    } else {
        format!("= {text}")
    };
    insert_after(src, &lines, last, &format!("{indent}{answer}"))
}

fn insert_after(src: &str, lines: &[Raw], line: usize, text: &str) -> Result<Change, EditError> {
    let eol = eol(src);
    let r = lines[line - 1];
    let edit = if r.next > r.end {
        // The line has an ending: the new line goes right after it.
        Replace {
            span: Span::new(r.next, r.next),
            text: format!("{text}{eol}"),
        }
    } else {
        // The last line, with no ending: give it one first.
        Replace {
            span: Span::new(r.end, r.end),
            text: format!("{eol}{text}"),
        }
    };
    Ok(Change::new(src, vec![edit]))
}

/// The smallest replacement turning one line into another.
fn line_diff(at: usize, old: &str, new: &str) -> Vec<Replace> {
    let prefix = old
        .char_indices()
        .zip(new.chars())
        .find(|((_, a), b)| a != b)
        .map_or(old.len().min(new.len()), |((i, _), _)| i);
    let prefix = (0..=prefix)
        .rev()
        .find(|&i| old.is_char_boundary(i) && new.is_char_boundary(i))
        .unwrap_or(0);
    let mut suffix = 0;
    let (ob, nb) = (old.as_bytes(), new.as_bytes());
    while suffix < ob.len() - prefix
        && suffix < nb.len() - prefix
        && ob[ob.len() - 1 - suffix] == nb[nb.len() - 1 - suffix]
    {
        suffix += 1;
    }
    while !old.is_char_boundary(old.len() - suffix) || !new.is_char_boundary(new.len() - suffix) {
        suffix -= 1;
    }
    vec![Replace {
        span: Span::new(at + prefix, at + old.len() - suffix),
        text: new[prefix..new.len() - suffix].to_string(),
    }]
}

// ----------------------------------------------------------------------------
// Formatting

/// `sprig fmt`: two spaces per level of the tree, typed dates written out
/// (§9.4), no trailing whitespace, and one line ending at the end of the file.
/// Line endings, a BOM, blank lines and comments otherwise stay as written.
pub fn format(src: &str, today: Date) -> Change {
    let lines = raw_lines(src);
    let kinds = syntax::lex(src);
    let doc = syntax::parse("", src).document;

    // The level each line belongs at: items by their depth, and notes and
    // answers at the depth of the item's children.
    let mut level: Vec<Option<usize>> = vec![None; lines.len()];
    let starts: Vec<usize> = lines.iter().map(|l| l.start).collect();
    fn walk(n: &Node, depth: usize, starts: &[usize], level: &mut [Option<usize>]) {
        for p in &n.notes {
            for l in &p.lines {
                // A note line knows its span, not its line number.
                let i = starts.partition_point(|&s| s <= l.span.start) - 1;
                level[i] = Some(depth);
            }
        }
        for a in &n.answers {
            level[a.line - 1] = Some(depth);
        }
        for c in &n.children {
            level[c.line - 1] = Some(depth);
            walk(c, depth + 1, starts, level);
        }
    }
    walk(&doc.root, 0, &starts, &mut level);
    // A root note is at level 0, unless it is the first line of a file
    // without a title, where no indentation would make it the title (§1.2).
    if doc.title.is_none()
        && let Some(first) = kinds
            .iter()
            .position(|l| !matches!(l.kind, syntax::LineKind::Blank | syntax::LineKind::Comment))
        && matches!(
            kinds[first].kind,
            syntax::LineKind::Note | syntax::LineKind::Escaped
        )
    {
        level[first] = Some(level[first].unwrap_or(0).max(1));
    }

    // A comment keeps its place relative to the lines above it: at the level
    // of the nearest one with the same indentation, or one level inside the
    // nearest item with less, and otherwise at the margin.
    let is_content = |i: usize| {
        !matches!(
            kinds[i].kind,
            syntax::LineKind::Blank | syntax::LineKind::Comment
        )
    };
    let comment_level = |i: usize| -> usize {
        let indent = lines[i].indent;
        for j in (0..i).rev().filter(|&j| is_content(j)) {
            let lvl = match kinds[j].kind {
                syntax::LineKind::Title => return 0,
                _ => level[j].unwrap_or(0),
            };
            if lines[j].indent == indent {
                return lvl;
            }
            // Only an item has an inside; past a shallower note, keep looking.
            if lines[j].indent < indent && matches!(kinds[j].kind, syntax::LineKind::Item(_)) {
                return lvl + 1;
            }
        }
        0
    };

    let mut edits = Vec::new();
    for (i, l) in lines.iter().enumerate() {
        let text = &src[l.text..l.end];
        let body = text.trim_end();
        if body.is_empty() {
            if l.end > l.start {
                edits.push(Replace {
                    span: Span::new(l.start, l.end),
                    text: String::new(),
                });
            }
            continue;
        }
        let kind = kinds[i].kind;
        let lvl = match kind {
            syntax::LineKind::Title => 0,
            syntax::LineKind::Comment => comment_level(i),
            _ => level[i].unwrap_or(0),
        };
        let new = if matches!(
            kind,
            syntax::LineKind::Title | syntax::LineKind::Item(_) | syntax::LineKind::Answer
        ) {
            syntax::expand_dates(body, today)
        } else {
            body.to_string()
        };
        let indented = format!("{}{new}", "  ".repeat(lvl));
        let old = &src[l.start..l.end];
        if old != indented {
            edits.extend(line_diff(l.start, old, &indented));
        }
    }

    // One line ending at the end: trailing blank lines go, a missing one comes.
    let eol = eol(src);
    let last_text = lines
        .iter()
        .rposition(|l| !src[l.start..l.end].trim().is_empty());
    match last_text {
        Some(i) => {
            let l = lines[i];
            // Edits to the blank lines after it are replaced by this one.
            edits.retain(|e| e.span.end <= l.end);
            if &src[l.end..] != eol {
                edits.push(Replace {
                    span: Span::new(l.end, src.len()),
                    text: eol.to_string(),
                });
            }
        }
        None => {
            // Nothing but blank lines: the file becomes empty, BOM and all
            // being the only thing kept.
            let body = if src.starts_with('\u{feff}') { 3 } else { 0 };
            edits.clear();
            if src.len() > body {
                edits.push(Replace {
                    span: Span::new(body, src.len()),
                    text: String::new(),
                });
            }
        }
    }
    Change::new(src, edits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> Date {
        Date::parse_iso("2026-10-02").unwrap()
    }

    #[test]
    fn set_mark_changes_one_byte() {
        let src = "Plan\n\n  - Paint  @jo\n";
        let c = set_mark(src, 3, Mark::Doing).unwrap();
        assert_eq!(c.source, "Plan\n\n  ~ Paint  @jo\n");
        assert_eq!(
            c.edits,
            [Replace {
                span: Span::new(8, 9),
                text: "~".into()
            }]
        );
        assert_eq!(set_mark(src, 1, Mark::Done), Err(EditError::NotAnItem(1)));
        assert_eq!(
            set_mark("P\n# G\n", 2, Mark::Done),
            Err(EditError::NoStatus(2))
        );
    }

    #[test]
    fn ticking_a_recurring_item_changes_only_the_date() {
        let src = "P\r\n- Numbers  every:fri  due:2026-10-02\r\n";
        let (c, moved) = tick(src, 2, today()).unwrap();
        assert_eq!(c.source, "P\r\n- Numbers  every:fri  due:2026-10-09\r\n");
        assert_eq!(moved.unwrap().to_string(), "2026-10-09");
        assert_eq!(c.edits.len(), 1);
        assert_eq!(c.edits[0].text, "9");
    }

    #[test]
    fn fields_are_changed_where_written_appended_or_removed() {
        let src = "P\n- Quote  due:2026-10-01  est:2h\n";
        assert_eq!(
            set_field(src, 2, "due", Some("2026-10-09")).unwrap().source,
            "P\n- Quote  due:2026-10-09  est:2h\n"
        );
        assert_eq!(
            set_field(src, 2, "where", Some("Mill Street"))
                .unwrap()
                .source,
            "P\n- Quote  due:2026-10-01  est:2h  where:\"Mill Street\"\n"
        );
        assert_eq!(
            set_field(src, 2, "due", None).unwrap().source,
            "P\n- Quote  est:2h\n"
        );
        assert!(set_field(src, 2, "nope", None).unwrap().is_empty());
    }

    #[test]
    fn children_go_last_at_their_siblings_indentation() {
        let src = "P\n- A\n    - a1\n      note\n\n- B\n";
        let c = add_item(src, 2, Place::Child, "- a2  @jo").unwrap();
        assert_eq!(
            c.source,
            "P\n- A\n    - a1\n      note\n    - a2  @jo\n\n- B\n"
        );
        let c = add_item(src, 6, Place::Child, "x b1").unwrap();
        assert_eq!(c.source, "P\n- A\n    - a1\n      note\n\n- B\n  x b1\n");
        let c = add_item(src, 2, Place::Sibling, "- A2").unwrap();
        assert_eq!(c.source, "P\n- A\n    - a1\n      note\n- A2\n\n- B\n");
        assert!(add_item(src, 2, Place::Child, "not an item").is_err());
        assert!(add_item(src, 2, Place::Child, "- a\n- b").is_err());
    }

    #[test]
    fn a_child_follows_the_last_childs_indentation() {
        let src = "R\n- Parent\n    - four in\n  - two in\n";
        let c = add_item(src, 2, Place::Child, "- added").unwrap();
        assert_eq!(
            c.source,
            "R\n- Parent\n    - four in\n  - two in\n  - added\n"
        );
    }

    #[test]
    fn a_last_line_without_an_ending_gets_one() {
        let c = add_item("P\n- A", 2, Place::Child, "- a").unwrap();
        assert_eq!(c.source, "P\n- A\n  - a");
    }

    #[test]
    fn answers_go_after_the_items_own_lines() {
        let src = "P\n? Name?\n  Poll: 31 to 24.\n  - Ask Teo\n";
        let c = add_answer(src, 2, "Kiln & Crumb.  @dana").unwrap();
        assert_eq!(
            c.source,
            "P\n? Name?\n  Poll: 31 to 24.\n  = Kiln & Crumb.  @dana\n  - Ask Teo\n"
        );
        let doc = syntax::parse("", &c.source).document;
        assert_eq!(doc.root.children[0].answers.len(), 1);
    }

    #[test]
    fn formatting_normalises_indentation_dates_and_the_end() {
        let src = "Plan  due:fri   \n\n- A\n     - a1  due:tomorrow\n\t\t\tnote under a1\n     // aside\n   - a2\n      // under a2\n\n\n";
        let c = format(src, today());
        assert_eq!(
            c.source,
            "Plan  due:2026-10-02\n\n- A\n  - a1  due:2026-10-03\n    note under a1\n  // aside\n  - a2\n    // under a2\n"
        );
        assert_eq!(format(&c.source, today()).source, c.source, "idempotent");
    }

    #[test]
    fn formatting_keeps_crlf_and_a_bom_and_an_untitled_first_note() {
        let src = "\u{feff}  first note\r\n- milk\r\n";
        let c = format(src, today());
        assert_eq!(c.source, "\u{feff}  first note\r\n- milk\r\n");
        assert!(c.is_empty());
        assert_eq!(format("", today()).source, "");
        assert_eq!(format("\n\n", today()).source, "");
        assert_eq!(format("P", today()).source, "P\n");
    }

    #[test]
    fn line_diffs_are_minimal_and_respect_characters() {
        assert_eq!(
            line_diff(10, "- Café  due:fri", "- Café  due:2026-10-02"),
            [Replace {
                span: Span::new(23, 26),
                text: "2026-10-02".into()
            }]
        );
        assert_eq!(line_diff(0, "é", "è")[0].span, Span::new(0, 2));
    }
}
