//! Reading one file into a tree (§1 to §3).
//!
//! The syntax tree holds only what the file says. Nothing here looks at another
//! file or at the date, so the parser can be tested on its own and a tree means
//! the same thing wherever it was built.

use crate::date;
use crate::diagnostic::{Code, Diagnostic, LineIndex, Span};
use crate::token::{self, Tokens};

/// The mark that starts an item line (§2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mark {
    Todo,
    Doing,
    Done,
    Dropped,
    Later,
    Question,
    Group,
    Graft,
}

impl Mark {
    fn from_char(c: char) -> Option<Mark> {
        Some(match c {
            '-' => Mark::Todo,
            '~' => Mark::Doing,
            'x' => Mark::Done,
            '/' => Mark::Dropped,
            '>' => Mark::Later,
            '?' => Mark::Question,
            '#' => Mark::Group,
            '+' => Mark::Graft,
            _ => return None,
        })
    }

    pub fn as_char(self) -> char {
        match self {
            Mark::Todo => '-',
            Mark::Doing => '~',
            Mark::Done => 'x',
            Mark::Dropped => '/',
            Mark::Later => '>',
            Mark::Question => '?',
            Mark::Group => '#',
            Mark::Graft => '+',
        }
    }

    /// Groups and grafts structure a plan; the rest are statuses (§5.1).
    pub fn is_structure(self) -> bool {
        matches!(self, Mark::Group | Mark::Graft)
    }
}

/// One line of a note, as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteLine {
    /// The note's text: the line without its indentation, and without the
    /// backslash of an escaped note (§2.8).
    pub text: String,
    /// The whole line after indentation, backslash included.
    pub span: Span,
}

/// Consecutive note lines of one item (§2.6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paragraph {
    pub lines: Vec<NoteLine>,
}

impl Paragraph {
    /// The lines joined with single spaces.
    pub fn text(&self) -> String {
        let parts: Vec<&str> = self.lines.iter().map(|l| l.text.as_str()).collect();
        parts.join(" ")
    }
}

/// A `= ` line (§2.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub tokens: Tokens,
    /// 1-based.
    pub line: usize,
    pub span: Span,
}

/// An item, or the root of a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    /// `None` for the root.
    pub mark: Option<Mark>,
    pub tokens: Tokens,
    /// 1-based line number. For the root, the title's line, or 0 when the
    /// file has no title.
    pub line: usize,
    /// The line after its indentation, up to and excluding the line ending
    /// and trailing whitespace. `None` for the root of a file without a title.
    pub span: Option<Span>,
    /// Indentation in columns (§3.1). The root's is irrelevant and reads 0.
    pub indent: usize,
    pub notes: Vec<Paragraph>,
    pub answers: Vec<Answer>,
    pub children: Vec<Node>,
}

/// A parsed file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    /// The title's text, when the file has a title (§1.2).
    pub title: Option<String>,
    pub root: Node,
}

/// A document and the problems found reading it.
#[derive(Clone, Debug)]
pub struct Parsed {
    pub document: Document,
    pub diagnostics: Vec<Diagnostic>,
}

struct Line<'a> {
    /// The text after indentation, trimmed at the end.
    content: &'a str,
    /// Where `content` starts in the file.
    at: usize,
    indent: usize,
    /// Any leading whitespace at all, which disqualifies a title (§1.2).
    indented: bool,
}

/// Splits the source into lines, dropping a leading BOM and the CR of CRLF
/// (§1.1), and measures each line's indentation (§3.1).
fn lines(src: &str) -> Vec<Line<'_>> {
    let body_at = if src.starts_with('\u{feff}') { 3 } else { 0 };
    let mut out = Vec::new();
    let mut at = body_at;
    for raw in src[body_at..].split('\n') {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let mut indent = 0;
        let mut ws = 0;
        for c in line.chars() {
            match c {
                ' ' => indent += 1,
                '\t' => indent += 2,
                _ => break,
            }
            ws += 1;
        }
        let rest = &line[ws..];
        let lead = rest.len() - rest.trim_start().len();
        let content = rest.trim();
        out.push(Line {
            content,
            at: at + ws + lead,
            indent,
            indented: ws + lead > 0,
        });
        at += raw.len() + 1;
    }
    out
}

/// A mark followed by whitespace or by the end of the line.
fn mark_of(content: &str) -> Option<(char, &str)> {
    let mut chars = content.chars();
    let c = chars.next()?;
    if !"-~x/>?#+=".contains(c) {
        return None;
    }
    let rest = chars.as_str();
    if rest.is_empty() || rest.starts_with(char::is_whitespace) {
        Some((c, rest.trim_start()))
    } else {
        None
    }
}

// The tree is built in an arena, because lines attach to whatever is on the
// indentation stack and Rust prefers indices to a stack of borrows.
struct Building {
    node: Node,
    kids: Vec<usize>,
}

/// What one line of a file is, for an editor that colours it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lexed {
    pub kind: LineKind,
    /// The line after indentation, as in a node's span.
    pub span: Span,
    /// The mark, the `=`, the backslash or the `//`, when the line has one.
    pub marker: Option<Span>,
    pub pieces: Vec<(token::Piece, Span)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Blank,
    Title,
    Comment,
    Note,
    /// A note written with a leading backslash (§2.8).
    Escaped,
    Item(Mark),
    Answer,
}

impl LineKind {
    pub fn as_str(self) -> &'static str {
        match self {
            LineKind::Blank => "blank",
            LineKind::Title => "title",
            LineKind::Comment => "comment",
            LineKind::Note => "note",
            LineKind::Escaped => "escaped",
            LineKind::Item(_) => "item",
            LineKind::Answer => "answer",
        }
    }
}

/// Classifies every line the way [`parse`] does, with the pieces of each, so
/// that an editor colours exactly what the parser reads.
pub fn lex(src: &str) -> Vec<Lexed> {
    let lines = lines(src);
    let title = title_line(&lines);
    lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let span = Span::new(l.at, l.at + l.content.len());
            let at = |n: usize| Span::new(l.at, l.at + n);
            let rest = |marker: usize| {
                let text = &l.content[marker..];
                let lead = text.len() - text.trim_start().len();
                (text.trim_start(), l.at + marker + lead)
            };
            if l.content.is_empty() {
                return Lexed {
                    kind: LineKind::Blank,
                    span,
                    marker: None,
                    pieces: Vec::new(),
                };
            }
            if Some(i) == title {
                return Lexed {
                    kind: LineKind::Title,
                    span,
                    marker: None,
                    pieces: token::pieces(l.content, l.at, false),
                };
            }
            if l.content.starts_with("//") {
                return Lexed {
                    kind: LineKind::Comment,
                    span,
                    marker: Some(span),
                    pieces: Vec::new(),
                };
            }
            if l.content.starts_with('\\') {
                let (text, base) = rest(1);
                return Lexed {
                    kind: LineKind::Escaped,
                    span,
                    marker: Some(at(1)),
                    pieces: token::pieces(text, base, true),
                };
            }
            match mark_of(l.content) {
                Some((c, _)) => {
                    let (text, base) = rest(c.len_utf8());
                    let kind = match Mark::from_char(c) {
                        Some(m) => LineKind::Item(m),
                        None => LineKind::Answer,
                    };
                    Lexed {
                        kind,
                        span,
                        marker: Some(at(c.len_utf8())),
                        pieces: token::pieces(text, base, false),
                    }
                }
                None => Lexed {
                    kind: LineKind::Note,
                    span,
                    marker: None,
                    pieces: token::pieces(l.content, l.at, true),
                },
            }
        })
        .collect()
}

/// Rewrites typed date shortcuts in one line to the stored form (§9.4), and
/// leaves every other byte alone. Only `due:` and `start:` on an item, an
/// answer or a title are dates; a note's text is never touched.
pub fn expand_dates(line: &str, today: crate::date::Date) -> String {
    // Behind a BOM of its own, so a BOM the line starts with is read as text,
    // not as the start of a file.
    let shifted = format!("\u{feff}{line}");
    let lexed = lex(&shifted);
    let Some(first) = lexed.first() else {
        return line.to_string();
    };
    let mut out = line.to_string();
    let pieces: Vec<(token::Piece, Span)> = first
        .pieces
        .iter()
        .map(|(p, s)| (*p, Span::new(s.start - 3, s.end - 3)))
        .collect();
    let pieces = &pieces;
    // From the end, so earlier spans stay valid as later ones change length.
    for pair in pieces.windows(2).rev() {
        let [(token::Piece::Key, key), (token::Piece::Value, value)] = pair else {
            continue;
        };
        let k = line[key.start..key.end].to_ascii_lowercase();
        let v = &line[value.start..value.end];
        if (k == "due:" || k == "start:")
            && crate::date::Date::parse_iso(v).is_none()
            && let Some(d) = crate::date::resolve(v, today)
        {
            out.replace_range(value.start..value.end, &d.to_string());
        }
    }
    out
}

/// One line, ticked (§10.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ticked {
    pub line: String,
    /// For a recurring item, the due date it moved to; it stays open.
    pub moved_to: Option<crate::date::Date>,
}

/// Ticks the item on `line`, a body line of a file: a done item reopens as to
/// do, an open recurring item moves its `due:` to the next occurrence and stays
/// to do (§10.4), and anything else becomes done. Only the mark and the due
/// date change. `None` when the line is not an item with a status.
pub fn tick(line: &str, today: crate::date::Date) -> Option<Ticked> {
    // Lexed under a stand-in title, so an unindented item reads as a body line.
    let lexed = lex(&format!("_\n{line}"));
    let l = lexed.get(1)?;
    let LineKind::Item(mark) = l.kind else {
        return None;
    };
    if mark.is_structure() {
        return None;
    }
    let shift = 2;
    let marker = l.marker?;
    let at = marker.start - shift;
    let mut out = line.to_string();
    let lifted = token::lift(&line[marker.end - shift..], marker.end - shift);
    let every = lifted.field("every").map(|f| f.value.clone());
    if mark != Mark::Done
        && let Some(every) = every
    {
        let due = lifted.field("due");
        let current = due.and_then(|f| crate::date::resolve(&f.value, today));
        if let Some(next) = crate::date::next_occurrence(current, &every, today) {
            match due {
                Some(f) => out.replace_range(f.span.start..f.span.end, &format!("due:{next}")),
                None => {
                    let end = out.trim_end().len();
                    out.replace_range(end.., &format!("  due:{next}"));
                }
            }
            out.replace_range(at..at + 1, "-");
            return Some(Ticked {
                line: out,
                moved_to: Some(next),
            });
        }
    }
    let to = if mark == Mark::Done { "-" } else { "x" };
    out.replace_range(at..at + 1, to);
    Some(Ticked {
        line: out,
        moved_to: None,
    })
}

/// §1.2: the first line that is not blank or a comment is the title, unless
/// it is indented or starts like an item or an answer.
fn title_line(lines: &[Line<'_>]) -> Option<usize> {
    let i = lines
        .iter()
        .position(|l| !l.content.is_empty() && !l.content.starts_with("//"))?;
    let l = &lines[i];
    (!l.indented && mark_of(l.content).is_none()).then_some(i)
}

/// Reads one file. `file` names it in diagnostics and nowhere else.
pub fn parse(file: &str, src: &str) -> Parsed {
    let index = LineIndex::new(src);
    let mut diagnostics = Vec::new();
    let lines = lines(src);

    let mut arena = vec![Building {
        node: Node {
            mark: None,
            tokens: Tokens::default(),
            line: 0,
            span: None,
            indent: 0,
            notes: Vec::new(),
            answers: Vec::new(),
            children: Vec::new(),
        },
        kids: Vec::new(),
    }];
    let mut title = None;

    let mut first_body = 0;
    if let Some(i) = title_line(&lines) {
        let l = &lines[i];
        let tokens = token::lift(l.content, l.at);
        title = Some(tokens.text.clone());
        let root = &mut arena[0].node;
        root.tokens = tokens;
        root.line = i + 1;
        root.span = Some(Span::new(l.at, l.at + l.content.len()));
        first_body = i + 1;
    }

    // The indentation stack holds arena indices; the root is never popped.
    let mut stack: Vec<usize> = vec![0];
    // Whether the last line that mattered was a note of this node, so the next
    // note continues its paragraph.
    let mut paragraph_open: Option<usize> = None;

    for (i, l) in lines.iter().enumerate().skip(first_body) {
        if l.content.is_empty() {
            paragraph_open = None;
            continue;
        }
        if l.content.starts_with("//") {
            continue;
        }
        while stack.len() > 1 && arena[*stack.last().unwrap()].node.indent >= l.indent {
            stack.pop();
        }
        let top = *stack.last().unwrap();
        let span = Span::new(l.at, l.at + l.content.len());
        let escaped = l.content.starts_with('\\');

        match (!escaped).then(|| mark_of(l.content)).flatten() {
            Some(('=', rest)) => {
                let at = l.at + l.content.len() - rest.len();
                arena[top].node.answers.push(Answer {
                    tokens: token::lift(rest, at),
                    line: i + 1,
                    span,
                });
                paragraph_open = None;
            }
            Some((c, rest)) => {
                let at = l.at + l.content.len() - rest.len();
                arena.push(Building {
                    node: Node {
                        mark: Mark::from_char(c),
                        tokens: token::lift(rest, at),
                        line: i + 1,
                        span: Some(span),
                        indent: l.indent,
                        notes: Vec::new(),
                        answers: Vec::new(),
                        children: Vec::new(),
                    },
                    kids: Vec::new(),
                });
                let id = arena.len() - 1;
                arena[top].kids.push(id);
                stack.push(id);
                paragraph_open = None;
            }
            None => {
                let text = if escaped { &l.content[1..] } else { l.content };
                let line = NoteLine {
                    text: text.to_string(),
                    span,
                };
                let notes = &mut arena[top].node.notes;
                match notes.last_mut() {
                    Some(p) if paragraph_open == Some(top) => p.lines.push(line),
                    _ => notes.push(Paragraph { lines: vec![line] }),
                }
                paragraph_open = Some(top);
            }
        }
    }

    let root = assemble(&mut arena, 0);
    check_tokens(file, &index, &root, &mut diagnostics);
    check_anchors(file, &index, &root, &mut diagnostics);
    diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
    Parsed {
        document: Document { title, root },
        diagnostics,
    }
}

fn assemble(arena: &mut [Building], id: usize) -> Node {
    let kids = std::mem::take(&mut arena[id].kids);
    let children = kids.into_iter().map(|k| assemble(arena, k)).collect();
    let mut node = std::mem::replace(
        &mut arena[id].node,
        Node {
            mark: None,
            tokens: Tokens::default(),
            line: 0,
            span: None,
            indent: 0,
            notes: Vec::new(),
            answers: Vec::new(),
            children: Vec::new(),
        },
    );
    node.children = children;
    node
}

/// Reads an estimate (§10.1) as hours: a number and `h`, `d` (8 hours) or `w`
/// (40 hours).
pub fn estimate_hours(value: &str) -> Option<f64> {
    let unit = value.chars().last()?;
    let hours = match unit.to_ascii_lowercase() {
        'h' => 1.0,
        'd' => 8.0,
        'w' => 40.0,
        _ => return None,
    };
    let number = &value[..value.len() - 1];
    let (whole, frac) = match number.split_once('.') {
        Some((w, f)) => (w, Some(f)),
        None => (number, None),
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !digits(whole) || frac.is_some_and(|f| !digits(f)) {
        return None;
    }
    number.parse::<f64>().ok().map(|n| n * hours)
}

// Problems a single line can have on its own: unreadable dates and estimates,
// several anchors, a graft with nothing to graft.
fn check_tokens(file: &str, index: &LineIndex, node: &Node, out: &mut Vec<Diagnostic>) {
    let mut check = |tokens: &Tokens| {
        for key in ["due", "start"] {
            if let Some(f) = tokens.field(key)
                && !date::readable(&f.value)
            {
                let mut d = index.diagnostic(
                    file,
                    Code::UnreadableDate,
                    f.span,
                    format!("{key}: \"{}\" is not a date; it is kept as text", f.value),
                );
                d.help =
                    Some("write YYYY-MM-DD, or a shortcut such as fri, +2w, oct3 or q4".into());
                out.push(d);
            }
        }
        if let Some(f) = tokens.field("est")
            && estimate_hours(&f.value).is_none()
        {
            let mut d = index.diagnostic(
                file,
                Code::UnreadableEstimate,
                f.span,
                format!("est:{} is not an estimate; it is ignored", f.value),
            );
            d.help = Some("write a number and a unit: est:4h, est:2d, est:1w".into());
            out.push(d);
        }
        if tokens.anchors.len() > 1 {
            let last = tokens.anchor().expect("there are several");
            out.push(index.diagnostic(
                file,
                Code::SeveralAnchors,
                last.span,
                format!(
                    "this item has {} anchors; only ^{} names it",
                    tokens.anchors.len(),
                    last.name
                ),
            ));
        }
    };
    if node.span.is_some() {
        check(&node.tokens);
    }
    for a in &node.answers {
        check(&a.tokens);
    }
    if node.mark == Some(Mark::Graft) && node.tokens.graft_link().is_none() {
        let span = node.span.expect("an item has a span");
        let mut d = index.diagnostic(
            file,
            Code::GraftWithoutLink,
            span,
            "this graft has no link, so it grafts nothing",
        );
        d.help =
            Some("name a file, like + [[kitchen]], or a branch, like + [[kitchen^oven]]".into());
        out.push(d);
    }
    for c in &node.children {
        check_tokens(file, index, c, out);
    }
}

fn check_anchors(file: &str, index: &LineIndex, root: &Node, out: &mut Vec<Diagnostic>) {
    let mut seen: Vec<(&str, usize)> = Vec::new();
    walk(root, &mut |n| {
        if let Some(a) = n.tokens.anchor() {
            match seen.iter().find(|(name, _)| *name == a.name) {
                Some((_, first)) => out.push(index.diagnostic(
                    file,
                    Code::DuplicateAnchor,
                    a.span,
                    format!(
                        "^{} is already the anchor of line {first}; references reach that one",
                        a.name
                    ),
                )),
                None => seen.push((&a.name, n.line)),
            }
        }
    });
}

/// Visits every item under `node` in document order, not `node` itself.
pub(crate) fn walk<'a>(node: &'a Node, f: &mut impl FnMut(&'a Node)) {
    for c in &node.children {
        f(c);
        walk(c, f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(src: &str) -> Document {
        parse("t.sprig", src).document
    }

    fn texts(n: &Node) -> Vec<String> {
        n.children.iter().map(|c| c.tokens.text.clone()).collect()
    }

    #[test]
    fn the_title_line_becomes_the_root() {
        let d = doc("// a comment\n\nOpen the bakery  @dana\n\n- Sign the lease\n");
        assert_eq!(d.title.as_deref(), Some("Open the bakery"));
        assert_eq!(d.root.line, 3);
        assert_eq!(d.root.tokens.people, ["dana"]);
        assert_eq!(texts(&d.root), ["Sign the lease"]);
    }

    #[test]
    fn a_file_whose_first_line_is_an_item_has_no_title() {
        let d = doc("- milk\n- bread\n");
        assert_eq!(d.title, None);
        assert_eq!(d.root.span, None);
        assert_eq!(texts(&d.root), ["milk", "bread"]);
    }

    #[test]
    fn an_indented_first_line_is_a_note_on_an_untitled_root() {
        let d = doc("  Not a title\n- milk\n");
        assert_eq!(d.title, None);
        assert_eq!(d.root.notes[0].text(), "Not a title");
    }

    #[test]
    fn a_mark_needs_whitespace_or_the_end_of_the_line_after_it() {
        let d = doc("Marks\n- to do\nx-ray\n#tag\n# group\n-\n");
        let marks: Vec<_> = d.root.children.iter().map(|c| c.mark).collect();
        assert_eq!(
            marks,
            [Some(Mark::Todo), Some(Mark::Group), Some(Mark::Todo)]
        );
        // Unindented, so they belong to the root, not to the item above.
        assert_eq!(d.root.notes[0].text(), "x-ray #tag");
    }

    #[test]
    fn indentation_is_relative() {
        let d = doc("R\n- Parent\n    - Child, four in\n  - Also a child, two in\n- Next\n");
        assert_eq!(texts(&d.root), ["Parent", "Next"]);
        assert_eq!(
            texts(&d.root.children[0]),
            ["Child, four in", "Also a child, two in"]
        );
    }

    #[test]
    fn notes_and_answers_attach_to_the_nearest_shallower_item() {
        let d = doc("R\n? Name\n  = Kiln  @dana\n  Poll: 31 to 24.\nA root note.\n");
        let q = &d.root.children[0];
        assert_eq!(q.answers[0].tokens.text, "Kiln");
        assert_eq!(q.answers[0].tokens.people, ["dana"]);
        assert_eq!(q.notes[0].text(), "Poll: 31 to 24.");
        assert_eq!(d.root.notes[0].text(), "A root note.");
    }

    #[test]
    fn blank_lines_start_paragraphs_and_comments_do_not() {
        let d = doc("R\n- Oven\n  Rondo 14.2k,\n  // aside\n  Mono 12.9k.\n\n  Ask @teo.\n");
        let n = &d.root.children[0];
        let paras: Vec<_> = n.notes.iter().map(Paragraph::text).collect();
        assert_eq!(paras, ["Rondo 14.2k, Mono 12.9k.", "Ask @teo."]);
        assert!(n.tokens.people.is_empty());
    }

    #[test]
    fn an_escaped_note_keeps_its_mark() {
        let d = doc("R\n- Pack\n  \\- not an item\n");
        assert_eq!(d.root.children[0].notes[0].text(), "- not an item");
        assert!(d.root.children[0].children.is_empty());
    }

    #[test]
    fn spans_slice_back_to_the_source() {
        let src = "Plan  @dana\r\n\r\n  - Sign  ^signed   \r\n    = Done.\r\n";
        let d = doc(src);
        let s = d.root.span.unwrap();
        assert_eq!(&src[s.start..s.end], "Plan  @dana");
        let n = &d.root.children[0];
        let s = n.span.unwrap();
        assert_eq!(&src[s.start..s.end], "- Sign  ^signed");
        let s = n.answers[0].span;
        assert_eq!(&src[s.start..s.end], "= Done.");
    }

    #[test]
    fn a_bom_is_not_indentation() {
        let d = doc("\u{feff}Plan\n- one\n");
        assert_eq!(d.title.as_deref(), Some("Plan"));
    }

    #[test]
    fn a_leading_tab_counts_as_two_spaces() {
        let d = doc("R\n- a\n\t- b\n  - c\n");
        assert_eq!(texts(&d.root.children[0]), ["b", "c"]);
        assert_eq!(d.root.children[0].children[0].indent, 2);
    }

    #[test]
    fn lexing_classifies_lines_as_the_parser_does() {
        let src = "// c\nPlan  @dana\n- Book  ^van\n  = yes\n  a note with @teo.\n  \\- escaped\n\n# Group\n";
        let kinds: Vec<&str> = lex(src).iter().map(|l| l.kind.as_str()).collect();
        assert_eq!(
            kinds,
            [
                "comment", "title", "item", "answer", "note", "escaped", "blank", "item", "blank"
            ]
        );
        let item = &lex(src)[2];
        let m = item.marker.unwrap();
        assert_eq!(&src[m.start..m.end], "-");
        let (_, a) = item.pieces[0];
        assert_eq!(&src[a.start..a.end], "^van");
    }

    #[test]
    fn typed_dates_expand_and_nothing_else_moves() {
        let today = crate::date::Date::parse_iso("2026-10-01").unwrap();
        assert_eq!(
            expand_dates("  - Call  due:fri   start:Today  every:fri  #x", today),
            "  - Call  due:2026-10-02   start:2026-10-01  every:fri  #x"
        );
        assert_eq!(
            expand_dates("- Ok  due:2026-10-09", today),
            "- Ok  due:2026-10-09"
        );
        assert_eq!(
            expand_dates("- Odd  due:soonish", today),
            "- Odd  due:soonish"
        );
        assert_eq!(
            expand_dates("  a note, due:fri", today),
            "  a note, due:fri"
        );
    }

    #[test]
    fn ticking_changes_the_mark_and_moves_a_recurring_due_date() {
        let today = crate::date::Date::parse_iso("2026-10-02").unwrap();
        let t = |l| tick(l, today).map(|t| t.line);
        assert_eq!(t("  - Paint  @jo").as_deref(), Some("  x Paint  @jo"));
        assert_eq!(t("x Paint").as_deref(), Some("- Paint"));
        assert_eq!(t("? Name?").as_deref(), Some("x Name?"));
        assert_eq!(
            t("  - Weekly numbers  every:fri  due:2026-10-02  !").as_deref(),
            Some("  - Weekly numbers  every:fri  due:2026-10-09  !")
        );
        assert_eq!(
            t("~ Water  every:3d").as_deref(),
            Some("- Water  every:3d  due:2026-10-05")
        );
        assert_eq!(
            t("x Water  every:3d").as_deref(),
            Some("- Water  every:3d"),
            "a done one reopens"
        );
        assert_eq!(t("# Group"), None);
        assert_eq!(t("+ [[kitchen]]"), None);
        assert_eq!(t("  a note"), None);
        assert_eq!(
            tick("- Water  every:3d", today)
                .unwrap()
                .moved_to
                .unwrap()
                .to_string(),
            "2026-10-05"
        );
    }

    #[test]
    fn estimates_read_as_hours() {
        assert_eq!(estimate_hours("2h"), Some(2.0));
        assert_eq!(estimate_hours("0.5d"), Some(4.0));
        assert_eq!(estimate_hours("3W"), Some(120.0));
        for bad in ["", "h", "2", "2x", ".5h", "1.h", "-1h", "1e3h", "２h"] {
            assert_eq!(estimate_hours(bad), None, "{bad}");
        }
    }

    #[test]
    fn line_problems_are_warnings_with_codes() {
        let p = parse(
            "t.sprig",
            "R\n- a  due:soonish  est:lots\n- b  ^x ^y\n- c  ^y\n+ kitchen\n",
        );
        let codes: Vec<_> = p.diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes, ["S0001", "S0002", "S0003", "S0004", "S0005"]);
        assert_eq!(
            p.diagnostics[0].to_string(),
            "t.sprig:2:6: warning[S0001]: due: \"soonish\" is not a date; it is kept as text"
        );
    }
}
