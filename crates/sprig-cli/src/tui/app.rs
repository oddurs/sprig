//! The TUI's state and what each key does to it.
//!
//! Nothing here performs a side effect. A key that would change a file
//! returns a [`Write`] saying which edit to make and what the line must still
//! say, and [`apply`] makes that edit to whatever text the file holds by
//! then. That split is what lets every key, every write and every race with
//! an editor be tested with no terminal and no disk underneath.

use std::collections::HashSet;

use sprig_core::edit::{self, Place};
use sprig_core::next;
use sprig_core::workspace::{Resolution, Resolved, Status, Tree};
use sprig_core::{Date, Mark, Workspace, agenda};

use crate::data::day as day_name;
use crate::verbs::item_line;
use crate::views::{glyph, hours, relative, wait_label};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Tree,
    Next,
    Agenda,
}

impl View {
    pub const ALL: [View; 3] = [View::Tree, View::Next, View::Agenda];

    pub fn name(self) -> &'static str {
        match self {
            View::Tree => "tree",
            View::Next => "next up",
            View::Agenda => "agenda",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Normal,
    /// Typing a filter; the view narrows as it is typed.
    Filter,
    /// Typing a new item, as a child of the cursor's item or after it.
    Add {
        child: bool,
        text: String,
    },
    Help,
}

/// A key, as the core understands one. The shell turns terminal events into
/// these; tests write them directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
}

/// The keys, as the help overlay lists them. A test drives each one.
pub const HELP: &[(&str, &str)] = &[
    ("j k  ↓ ↑", "move down and up"),
    (
        "h l  ← →",
        "fold and unfold; h on a leaf goes to its parent",
    ),
    ("g G", "first and last item"),
    (
        "x",
        "tick: done, or reopen; a recurring item moves its due date",
    ),
    ("~", "mark as doing"),
    ("a A", "add a sibling after the item, or a child under it"),
    ("e", "open $EDITOR at the item's line"),
    ("u", "undo this session's last write"),
    ("1 2 3", "tree, next up, agenda"),
    ("/", "filter by text; Esc clears"),
    ("r", "read the files again"),
    ("?", "this help"),
    ("q", "quit"),
];

/// Where an item is: enough to find it again after the file changes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Spot {
    pub file: String,
    pub line: usize,
}

/// One line of a view.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// `None` for a heading, which the cursor skips.
    pub at: Option<Spot>,
    pub depth: usize,
    pub glyph: char,
    pub label: String,
    /// Dim text after the label: where an item is, in the flat views.
    pub context: String,
    /// Right-aligned: progress, due date, people.
    pub info: String,
    pub status: Status,
    pub blocked: bool,
    pub branch: bool,
    pub folded: bool,
    pub heading: bool,
}

/// What the cursor's item looks like, for the detail pane.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Detail {
    pub label: String,
    pub place: String,
    pub lines: Vec<(String, String)>,
    pub waits: Vec<String>,
    pub notes: Vec<String>,
}

/// An edit to make, and what the file must still say for it to be the edit
/// that was meant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Write {
    pub file: String,
    pub op: Op,
    pub expect: Expect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    Tick {
        line: usize,
    },
    Mark {
        line: usize,
        mark: Mark,
    },
    Add {
        line: usize,
        place: Place,
        text: String,
    },
    /// Put back the text from before this session's last write.
    Restore {
        text: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expect {
    /// The line still says this.
    Line { line: usize, text: String },
    /// The whole file is still what the TUI last wrote.
    Whole(String),
}

/// What the shell around [`App`] must do after a key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    Reload,
    Write(Write),
    /// Suspend, open the editor at this line, and read the files again.
    Edit {
        file: String,
        line: usize,
    },
}

/// Makes `w` to `disk`, the file's text as it is now: the new text and the
/// line to report. Refuses, changing nothing, when the file no longer says
/// what the TUI saw, because then the edit would land on something else.
pub fn apply(disk: &str, w: &Write, today: Date) -> Result<(String, usize), String> {
    match &w.expect {
        Expect::Line { line, text } => {
            let now = disk.lines().nth(line - 1).map(|l| l.trim_end_matches('\r'));
            if now != Some(text.as_str()) {
                return Err(format!(
                    "{}:{line} changed on disk since it was read; nothing was written",
                    w.file
                ));
            }
        }
        Expect::Whole(text) => {
            if disk != text {
                return Err(format!(
                    "{} changed on disk since the last write, so it cannot be undone; nothing was written",
                    w.file
                ));
            }
        }
    }
    let err = |e: edit::EditError| format!("{}: {e}", w.file);
    match &w.op {
        Op::Tick { line } => Ok((edit::tick(disk, *line, today).map_err(err)?.0.source, *line)),
        Op::Mark { line, mark } => Ok((
            edit::set_mark(disk, *line, *mark).map_err(err)?.source,
            *line,
        )),
        Op::Add { line, place, text } => {
            let c = edit::add_item(disk, *line, *place, text).map_err(err)?;
            let e = &c.edits[0];
            let at = disk[..e.span.start].matches('\n').count() + 1;
            let at = if e.text.starts_with(['\n', '\r']) {
                at + 1
            } else {
                at
            };
            Ok((c.source, at))
        }
        Op::Restore { text } => Ok((text.clone(), 0)),
    }
}

/// One write this session made, kept so `u` can put it back.
#[derive(Clone, Debug)]
struct Undo {
    file: String,
    before: String,
    after: String,
}

/// What to keep the cursor on across a reload: its anchor if it has one,
/// else its text, else its line.
#[derive(Clone, Debug, Default)]
struct Track {
    file: String,
    line: usize,
    anchor: Option<String>,
    label: String,
}

pub struct App {
    pub today: Date,
    ws: Workspace,
    entries: Option<Vec<String>>,
    res: Resolution,
    pub view: View,
    pub mode: Mode,
    pub filter: String,
    folded: HashSet<Spot>,
    pub rows: Vec<Row>,
    pub cursor: usize,
    /// What the last action did, for the status line.
    pub flash: Option<String>,
    undo: Vec<Undo>,
}

impl App {
    /// `entries` are the files named on the command line, or `None` for a
    /// folder, whose top-level plans are the entries.
    pub fn new(ws: Workspace, entries: Option<Vec<String>>, today: Date) -> App {
        let res = ws.resolve(today);
        let mut app = App {
            today,
            ws,
            entries,
            res,
            view: View::Tree,
            mode: Mode::Normal,
            filter: String::new(),
            folded: HashSet::new(),
            rows: Vec::new(),
            cursor: 0,
            flash: None,
            undo: Vec::new(),
        };
        app.rebuild();
        app.cursor = app.first_item(0, 1).unwrap_or(0);
        app
    }

    /// The files as they are now on disk, keeping the cursor on its item.
    pub fn reload(&mut self, ws: Workspace) {
        let track = self.track();
        self.ws = ws;
        self.res = self.ws.resolve(self.today);
        self.rebuild();
        if let Some(t) = track {
            self.cursor = self
                .find(&t)
                .unwrap_or(self.cursor.min(self.rows.len().saturating_sub(1)));
        }
    }

    /// The text of a file as the TUI last read it.
    pub fn source(&self, file: &str) -> Option<&str> {
        self.ws.files().find(|(k, _)| *k == file).map(|(_, s)| s)
    }

    /// The shell made a write: remember it for undo, unless it was an undo.
    pub fn written(&mut self, w: &Write, before: String, after: String, line: usize) {
        let what = after
            .lines()
            .nth(line.max(1) - 1)
            .unwrap_or("")
            .trim()
            .to_string();
        if matches!(w.op, Op::Restore { .. }) {
            self.undo.pop();
            self.flash = Some(format!("undone in {}", w.file));
        } else {
            let old = before
                .lines()
                .nth(line.max(1) - 1)
                .unwrap_or("")
                .trim()
                .to_string();
            self.flash = Some(match w.op {
                Op::Add { .. } => format!("{}:{line}  + {what}", w.file),
                _ => format!("{}:{line}  {old} → {what}", w.file),
            });
            self.undo.push(Undo {
                file: w.file.clone(),
                before,
                after,
            });
        }
    }

    pub fn failed(&mut self, why: String) {
        self.flash = Some(why);
    }

    /// The status line's counts, over the entry trees.
    pub fn counts(&self) -> (usize, usize, usize, usize) {
        let trees = self.trees();
        let n = next::next(&trees, None);
        let (done, total) = trees.iter().fold((0, 0), |(d, t), tr| {
            (d + tr.root.progress.done, t + tr.root.progress.total)
        });
        (done, total, n.ready.len(), n.waiting.len())
    }

    pub fn errors(&self) -> usize {
        self.res
            .diagnostics
            .iter()
            .filter(|d| d.severity() == sprig_core::Severity::Error)
            .count()
    }

    pub fn key(&mut self, k: Key) -> Action {
        match self.mode.clone() {
            Mode::Help => {
                self.mode = Mode::Normal;
                if k == Key::Char('q') {
                    return Action::Quit;
                }
                Action::None
            }
            Mode::Filter => {
                match k {
                    Key::Esc => {
                        self.filter.clear();
                        self.mode = Mode::Normal;
                    }
                    Key::Enter => self.mode = Mode::Normal,
                    Key::Backspace => {
                        self.filter.pop();
                    }
                    Key::Char(c) => self.filter.push(c),
                    Key::Down => self.step(1),
                    Key::Up => self.step(-1),
                    _ => {}
                }
                self.refilter();
                Action::None
            }
            Mode::Add { child, mut text } => match k {
                Key::Esc => {
                    self.mode = Mode::Normal;
                    Action::None
                }
                Key::Enter => {
                    self.mode = Mode::Normal;
                    if text.trim().is_empty() {
                        return Action::None;
                    }
                    let Some((at, line_text)) = self.current() else {
                        return Action::None;
                    };
                    let place = if child { Place::Child } else { Place::Sibling };
                    Action::Write(Write {
                        file: at.file.clone(),
                        op: Op::Add {
                            line: at.line,
                            place,
                            text: item_line(&text, self.today),
                        },
                        expect: Expect::Line {
                            line: at.line,
                            text: line_text,
                        },
                    })
                }
                Key::Backspace => {
                    text.pop();
                    self.mode = Mode::Add { child, text };
                    Action::None
                }
                Key::Char(c) => {
                    text.push(c);
                    self.mode = Mode::Add { child, text };
                    Action::None
                }
                _ => Action::None,
            },
            Mode::Normal => self.normal(k),
        }
    }

    fn normal(&mut self, k: Key) -> Action {
        self.flash = None;
        match k {
            Key::Char('q') => return Action::Quit,
            Key::Char('j') | Key::Down => self.step(1),
            Key::Char('k') | Key::Up => self.step(-1),
            Key::Char('g') | Key::Home => self.cursor = self.first_item(0, 1).unwrap_or(0),
            Key::Char('G') | Key::End => {
                self.cursor = self
                    .first_item(self.rows.len().saturating_sub(1), -1)
                    .unwrap_or(0)
            }
            Key::Char('h') | Key::Left => self.fold_or_up(),
            Key::Char('l') | Key::Right | Key::Enter => self.unfold(),
            Key::Char(c @ '1'..='3') => {
                self.view = View::ALL[(c as u8 - b'1') as usize];
                self.rebuild();
                self.cursor = self.first_item(0, 1).unwrap_or(0);
            }
            Key::Char('/') => self.mode = Mode::Filter,
            Key::Esc => {
                self.filter.clear();
                self.refilter();
            }
            Key::Char('?') => self.mode = Mode::Help,
            Key::Char('r') => return Action::Reload,
            Key::Char('a') | Key::Char('A') if self.current().is_some() => {
                self.mode = Mode::Add {
                    child: k == Key::Char('A'),
                    text: String::new(),
                }
            }
            Key::Char('x') => {
                if let Some((at, text)) = self.current() {
                    return Action::Write(Write {
                        file: at.file.clone(),
                        op: Op::Tick { line: at.line },
                        expect: Expect::Line {
                            line: at.line,
                            text,
                        },
                    });
                }
            }
            Key::Char('~') => {
                if let Some((at, text)) = self.current() {
                    return Action::Write(Write {
                        file: at.file.clone(),
                        op: Op::Mark {
                            line: at.line,
                            mark: Mark::Doing,
                        },
                        expect: Expect::Line {
                            line: at.line,
                            text,
                        },
                    });
                }
            }
            Key::Char('e') => {
                if let Some((at, _)) = self.current() {
                    return Action::Edit {
                        file: at.file,
                        line: at.line,
                    };
                }
            }
            Key::Char('u') => match self.undo.last() {
                Some(u) => {
                    return Action::Write(Write {
                        file: u.file.clone(),
                        op: Op::Restore {
                            text: u.before.clone(),
                        },
                        expect: Expect::Whole(u.after.clone()),
                    });
                }
                None => self.flash = Some("nothing to undo".into()),
            },
            _ => {}
        }
        Action::None
    }

    /// The cursor's item and its line as the file says it now.
    fn current(&self) -> Option<(Spot, String)> {
        let at = self.rows.get(self.cursor)?.at.clone()?;
        let text = self
            .source(&at.file)?
            .lines()
            .nth(at.line.checked_sub(1)?)?
            .trim_end_matches('\r')
            .to_string();
        Some((at, text))
    }

    /// The nearest selectable row from `from`, moving by `dir`.
    fn first_item(&self, from: usize, dir: isize) -> Option<usize> {
        let mut i = from as isize;
        while i >= 0 && (i as usize) < self.rows.len() {
            if self.rows[i as usize].at.is_some() {
                return Some(i as usize);
            }
            i += dir;
        }
        None
    }

    fn step(&mut self, dir: isize) {
        let next = self.cursor as isize + dir;
        if next < 0 || next as usize >= self.rows.len() {
            return;
        }
        if let Some(i) = self.first_item(next as usize, dir) {
            self.cursor = i;
        }
    }

    fn fold_or_up(&mut self) {
        let Some(row) = self.rows.get(self.cursor).cloned() else {
            return;
        };
        if self.view != View::Tree {
            return;
        }
        if row.branch && !row.folded {
            if let Some(at) = row.at {
                self.folded.insert(at);
                self.rebuild();
            }
            return;
        }
        // To the parent: the nearest row above that sits shallower.
        if let Some(i) = (0..self.cursor)
            .rev()
            .find(|&i| self.rows[i].depth < row.depth)
        {
            self.cursor = i;
        }
    }

    fn unfold(&mut self) {
        if let Some(at) = self.rows.get(self.cursor).and_then(|r| r.at.clone())
            && self.folded.remove(&at)
        {
            self.rebuild();
        }
    }

    fn refilter(&mut self) {
        let track = self.track();
        self.rebuild();
        self.cursor = track
            .and_then(|t| self.find(&t))
            .or_else(|| self.first_item(0, 1))
            .unwrap_or(0);
    }

    fn track(&self) -> Option<Track> {
        let at = self.rows.get(self.cursor)?.at.clone()?;
        let n = self.node(&at)?;
        Some(Track {
            anchor: n.tokens.anchor().map(|a| a.name.clone()),
            label: n.label.clone(),
            file: at.file,
            line: at.line,
        })
    }

    fn find(&self, t: &Track) -> Option<usize> {
        let in_file = |r: &Row| r.at.as_ref().is_some_and(|a| a.file == t.file);
        let rows = || self.rows.iter().enumerate().filter(|(_, r)| in_file(r));
        if let Some(a) = &t.anchor
            && let Some((i, _)) = rows().find(|(_, r)| {
                let n = r.at.as_ref().and_then(|at| self.node(at));
                n.and_then(|n| n.tokens.anchor())
                    .is_some_and(|x| &x.name == a)
            })
        {
            return Some(i);
        }
        if let Some((i, _)) = rows().find(|(_, r)| r.label == t.label) {
            return Some(i);
        }
        rows()
            .min_by_key(|(_, r)| {
                r.at.as_ref()
                    .map_or(usize::MAX, |a| a.line.abs_diff(t.line))
            })
            .map(|(i, _)| i)
    }

    fn trees(&self) -> Vec<&Tree> {
        match &self.entries {
            Some(files) => files.iter().filter_map(|f| self.res.tree(f)).collect(),
            None => self
                .res
                .roots()
                .into_iter()
                .filter_map(|f| self.res.tree(f))
                .collect(),
        }
    }

    /// The node written on a line, found in the tree of its own file.
    pub fn node(&self, at: &Spot) -> Option<&Resolved> {
        fn walk(n: &Resolved, line: usize) -> Option<&Resolved> {
            if n.line == line {
                return Some(n);
            }
            n.children
                .iter()
                .filter(|c| !c.grafted)
                .find_map(|c| walk(c, line))
        }
        walk(&self.res.tree(&at.file)?.root, at.line)
    }

    pub fn detail(&self) -> Option<Detail> {
        let at = self.rows.get(self.cursor)?.at.clone()?;
        let n = self.node(&at)?;
        let mut lines = vec![("status".to_string(), status_word(n).to_string())];
        if !n.people.is_empty() {
            let people: Vec<String> = n.people.iter().map(|p| format!("@{p}")).collect();
            lines.push(("owner".into(), people.join(" ")));
        }
        if let Some(d) = n.due {
            lines.push((
                "due".into(),
                format!("{d} ({})", relative(self.today.days_until(d))),
            ));
        }
        if let Some(d) = n.start {
            lines.push(("start".into(), d.to_string()));
        }
        if let Some(e) = n.tokens.field("every") {
            lines.push(("every".into(), e.value.clone()));
        }
        if n.estimate > 0.0 {
            lines.push(("estimate".into(), hours(n.estimate)));
        }
        if !n.is_leaf() {
            let p = n.progress;
            lines.push(("progress".into(), format!("{} of {} done", p.done, p.total)));
        }
        let waits = n.waits_on.iter().map(wait_label).collect();
        let notes = self
            .res
            .parsed
            .get(&at.file)
            .and_then(|p| {
                let mut found = None;
                sprig_core::syntax::walk(&p.document.root, &mut |s| {
                    if s.line == at.line {
                        found = Some(s.notes.iter().map(|p| p.text()).collect::<Vec<_>>());
                    }
                });
                found
            })
            .unwrap_or_default();
        Some(Detail {
            label: n.label.clone(),
            place: format!("{}:{}", at.file, at.line),
            lines,
            waits,
            notes,
        })
    }

    fn rebuild(&mut self) {
        let needle = self.filter.to_lowercase();
        let hit = |label: &str| needle.is_empty() || label.to_lowercase().contains(&needle);
        let mut rows = Vec::new();
        match self.view {
            View::Tree => {
                for t in self.trees() {
                    tree_rows(
                        &t.root,
                        0,
                        &self.folded,
                        &hit,
                        !needle.is_empty(),
                        self.today,
                        &mut rows,
                    );
                }
            }
            View::Next => {
                let trees = self.trees();
                for e in next::next(&trees, None).ready {
                    if !hit(&e.node.label) {
                        continue;
                    }
                    let mut r = row(e.node, 0, self.today);
                    r.context = e.path.join(" › ");
                    rows.push(r);
                }
            }
            View::Agenda => {
                let dated = agenda::dated(&self.res);
                let w = agenda::window(dated, self.today, 14);
                let section = |name: String, items: &[agenda::Dated], rows: &mut Vec<Row>| {
                    let items: Vec<&agenda::Dated> =
                        items.iter().filter(|d| hit(&d.node.label)).collect();
                    if items.is_empty() {
                        return;
                    }
                    rows.push(heading(name));
                    for d in items {
                        let mut r = row(d.node, 1, self.today);
                        r.info = d.every.map(|e| format!("every:{e}")).unwrap_or_default();
                        r.context = d.node.file.clone();
                        rows.push(r);
                    }
                };
                section("Late".into(), &w.late, &mut rows);
                for (d, items) in &w.days {
                    let name = match self.today.days_until(*d) {
                        0 => format!("{} · today", day_name(*d)),
                        1 => format!("{} · tomorrow", day_name(*d)),
                        _ => day_name(*d),
                    };
                    section(name, items, &mut rows);
                }
            }
        }
        self.rows = rows;
        self.cursor = self.cursor.min(self.rows.len().saturating_sub(1));
    }
}

fn heading(label: String) -> Row {
    Row {
        at: None,
        depth: 0,
        glyph: ' ',
        label,
        context: String::new(),
        info: String::new(),
        status: Status::Todo,
        blocked: false,
        branch: false,
        folded: false,
        heading: true,
    }
}

fn row(n: &Resolved, depth: usize, today: Date) -> Row {
    let mut info = Vec::new();
    if !n.is_leaf() {
        info.push(format!("{}/{}", n.progress.done, n.progress.total));
    }
    if let Some(d) = n.due
        && n.status.is_open()
    {
        info.push(relative(today.days_until(d)));
    }
    // Only the people it names: inherited ones would repeat down the tree.
    if let Some(p) = n.tokens.people.first() {
        info.push(format!("@{p}"));
    }
    Row {
        // A grafted copy is found, and written, where its file writes it.
        at: (n.line != 0).then(|| Spot {
            file: n.file.clone(),
            line: n.line,
        }),
        depth,
        glyph: glyph(n),
        label: n.label.clone(),
        context: String::new(),
        info: info.join("  "),
        status: n.status,
        blocked: n.blocked && n.status.is_open(),
        branch: !n.is_leaf(),
        folded: false,
        heading: n.line == 0,
    }
}

/// Whether `n` or anything under it matches, so a filtered tree keeps the
/// ancestors of every match.
fn matches(n: &Resolved, hit: &dyn Fn(&str) -> bool) -> bool {
    hit(&n.label) || n.children.iter().any(|c| matches(c, hit))
}

fn tree_rows(
    n: &Resolved,
    depth: usize,
    folded: &HashSet<Spot>,
    hit: &dyn Fn(&str) -> bool,
    filtering: bool,
    today: Date,
    out: &mut Vec<Row>,
) {
    if filtering && !matches(n, hit) {
        return;
    }
    let mut r = row(n, depth, today);
    if n.line == 0 {
        // A file without a title: its name stands for it.
        r.heading = true;
    }
    let is_folded = !filtering && r.at.as_ref().is_some_and(|a| folded.contains(a));
    r.folded = is_folded && r.branch;
    out.push(r);
    if is_folded {
        return;
    }
    for c in &n.children {
        tree_rows(c, depth + 1, folded, hit, filtering, today, out);
    }
}

fn status_word(n: &Resolved) -> &'static str {
    match n.status {
        _ if n.blocked && n.status.is_open() => "blocked",
        Status::Todo => "to do",
        Status::Doing => "doing",
        Status::Done => "done",
        Status::Dropped => "dropped",
        Status::Later => "later",
        Status::Question if n.answered => "answered",
        Status::Question => "question",
        _ => "open",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAN: &str = "\
Plan  @me

- Design  ^design
  x Sketch
  - Plumbing  due:2026-10-05
- Build  after:^design
  - Walls
? Colour?  @jo
- Weekly  due:2026-10-02  every:fri
";

    fn today() -> Date {
        Date::new(2026, 10, 1).unwrap()
    }

    fn ws(src: &str) -> Workspace {
        let mut ws = Workspace::new();
        ws.insert("plan.sprig", src);
        ws
    }

    fn app() -> App {
        App::new(ws(PLAN), None, today())
    }

    fn labels(a: &App) -> Vec<&str> {
        a.rows.iter().map(|r| r.label.as_str()).collect()
    }

    fn at(a: &App) -> &str {
        &a.rows[a.cursor].label
    }

    fn to(a: &mut App, label: &str) {
        a.cursor = a.rows.iter().position(|r| r.label == label).unwrap();
    }

    /// Applies a write the way the shell does, and reloads.
    fn perform(a: &mut App, action: Action) -> String {
        let Action::Write(w) = action else {
            panic!("{action:?} is not a write")
        };
        let before = a.source(&w.file).unwrap().to_string();
        let (after, line) = apply(&before, &w, today()).unwrap();
        let mut next = a.ws.clone();
        next.insert(w.file.clone(), after.clone());
        a.reload(next);
        a.written(&w, before, after.clone(), line);
        after
    }

    #[test]
    fn j_k_and_the_arrows_move_down_and_up() {
        let mut a = app();
        assert_eq!(at(&a), "Plan");
        a.key(Key::Char('j'));
        assert_eq!(at(&a), "Design");
        a.key(Key::Down);
        assert_eq!(at(&a), "Sketch");
        a.key(Key::Char('k'));
        a.key(Key::Up);
        assert_eq!(at(&a), "Plan");
        // At the top, up stays put.
        a.key(Key::Up);
        assert_eq!(at(&a), "Plan");
    }

    #[test]
    fn h_folds_a_branch_and_l_unfolds_it_and_h_on_a_leaf_goes_to_its_parent() {
        let mut a = app();
        to(&mut a, "Design");
        a.key(Key::Char('h'));
        assert!(!labels(&a).contains(&"Sketch"));
        assert!(a.rows[a.cursor].folded);
        a.key(Key::Char('l'));
        assert!(labels(&a).contains(&"Sketch"));
        a.key(Key::Left);
        a.key(Key::Right);
        assert!(labels(&a).contains(&"Sketch"));
        to(&mut a, "Plumbing");
        a.key(Key::Char('h'));
        assert_eq!(at(&a), "Design");
    }

    #[test]
    fn g_and_capital_g_go_to_the_first_and_last_item() {
        let mut a = app();
        a.key(Key::Char('G'));
        assert_eq!(at(&a), "Weekly");
        a.key(Key::Char('g'));
        assert_eq!(at(&a), "Plan");
        a.key(Key::End);
        a.key(Key::Home);
        assert_eq!(at(&a), "Plan");
    }

    #[test]
    fn x_ticks_the_item_and_its_dependents_unblock_in_the_same_frame() {
        let mut a = app();
        let build = |a: &App| a.rows.iter().find(|r| r.label == "Build").unwrap().blocked;
        assert!(build(&a));
        to(&mut a, "Plumbing");
        let action = a.key(Key::Char('x'));
        assert_eq!(
            action,
            Action::Write(Write {
                file: "plan.sprig".into(),
                op: Op::Tick { line: 5 },
                expect: Expect::Line {
                    line: 5,
                    text: "  - Plumbing  due:2026-10-05".into()
                },
            })
        );
        let after = perform(&mut a, action);
        assert!(after.contains("  x Plumbing  due:2026-10-05\n"));
        assert_eq!(at(&a), "Plumbing");
        assert_eq!(
            a.flash.as_deref(),
            Some("plan.sprig:5  - Plumbing  due:2026-10-05 → x Plumbing  due:2026-10-05")
        );
        // A parent keeps its written status (§5.6), so Build still waits on
        // Design until Design itself is ticked; then it waits on nothing.
        assert!(build(&a));
        to(&mut a, "Design");
        let tick = a.key(Key::Char('x'));
        perform(&mut a, tick);
        assert!(!build(&a));
    }

    #[test]
    fn x_on_a_recurring_item_moves_its_due_date() {
        let mut a = app();
        to(&mut a, "Weekly");
        let action = a.key(Key::Char('x'));
        let after = perform(&mut a, action);
        assert!(
            after.contains("- Weekly  due:2026-10-09  every:fri\n"),
            "{after}"
        );
    }

    #[test]
    fn tilde_marks_the_item_as_doing() {
        let mut a = app();
        to(&mut a, "Walls");
        let action = a.key(Key::Char('~'));
        assert!(perform(&mut a, action).contains("  ~ Walls\n"));
        assert_eq!(a.rows[a.cursor].glyph, '~');
    }

    #[test]
    fn a_adds_a_sibling_after_and_capital_a_a_child_under() {
        let mut a = app();
        to(&mut a, "Design");
        a.key(Key::Char('A'));
        assert_eq!(
            a.mode,
            Mode::Add {
                child: true,
                text: String::new()
            }
        );
        for c in "Wiring due:fri".chars() {
            a.key(Key::Char(c));
        }
        a.key(Key::Backspace);
        a.key(Key::Char('i'));
        let action = a.key(Key::Enter);
        let Action::Write(w) = &action else { panic!() };
        assert_eq!(
            w.op,
            Op::Add {
                line: 3,
                place: Place::Child,
                text: "- Wiring due:2026-10-02".into()
            }
        );
        let after = perform(&mut a, action);
        assert!(
            after.contains("  - Plumbing  due:2026-10-05\n  - Wiring due:2026-10-02\n- Build"),
            "{after}"
        );
        assert_eq!(
            a.flash.as_deref(),
            Some("plan.sprig:6  + - Wiring due:2026-10-02")
        );

        to(&mut a, "Build");
        a.key(Key::Char('a'));
        a.key(Key::Char('P'));
        let add = a.key(Key::Enter);
        let after = perform(&mut a, add);
        assert!(after.contains("  - Walls\n- P\n? Colour?"), "{after}");

        // Esc, or nothing typed, adds nothing.
        a.key(Key::Char('a'));
        a.key(Key::Esc);
        assert_eq!(a.mode, Mode::Normal);
        a.key(Key::Char('a'));
        assert_eq!(a.key(Key::Enter), Action::None);
    }

    #[test]
    fn e_opens_the_editor_at_the_items_line() {
        let mut a = app();
        to(&mut a, "Walls");
        assert_eq!(
            a.key(Key::Char('e')),
            Action::Edit {
                file: "plan.sprig".into(),
                line: 7
            }
        );
    }

    #[test]
    fn u_undoes_this_sessions_writes_in_reverse() {
        let mut a = app();
        assert_eq!(a.key(Key::Char('u')), Action::None);
        assert_eq!(a.flash.as_deref(), Some("nothing to undo"));
        to(&mut a, "Walls");
        let tick = a.key(Key::Char('x'));
        perform(&mut a, tick);
        let doing = a.key(Key::Char('~'));
        perform(&mut a, doing);
        let undo = a.key(Key::Char('u'));
        assert!(perform(&mut a, undo).contains("  x Walls\n"));
        let undo = a.key(Key::Char('u'));
        assert_eq!(perform(&mut a, undo), PLAN);
        assert_eq!(a.key(Key::Char('u')), Action::None);
    }

    #[test]
    fn one_two_and_three_switch_to_the_tree_next_up_and_the_agenda() {
        let mut a = app();
        a.key(Key::Char('2'));
        assert_eq!(a.view, View::Next);
        // Ready work only: Build waits on Design.
        assert!(!labels(&a).contains(&"Build"));
        assert!(labels(&a).contains(&"Plumbing"));
        a.key(Key::Char('3'));
        assert_eq!(a.view, View::Agenda);
        assert_eq!(
            labels(&a),
            ["Fri 2 Oct · tomorrow", "Weekly", "Mon 5 Oct", "Plumbing"]
        );
        // The cursor skips headings.
        assert_eq!(at(&a), "Weekly");
        a.key(Key::Char('j'));
        assert_eq!(at(&a), "Plumbing");
        a.key(Key::Char('1'));
        assert_eq!(a.view, View::Tree);
    }

    #[test]
    fn slash_filters_as_you_type_keeping_ancestors_and_esc_clears() {
        let mut a = app();
        a.key(Key::Char('/'));
        assert_eq!(a.mode, Mode::Filter);
        for c in "wal".chars() {
            a.key(Key::Char(c));
        }
        assert_eq!(labels(&a), ["Plan", "Build", "Walls"]);
        a.key(Key::Enter);
        assert_eq!(a.mode, Mode::Normal);
        assert_eq!(labels(&a), ["Plan", "Build", "Walls"]);
        a.key(Key::Esc);
        assert!(labels(&a).contains(&"Sketch"));
        a.key(Key::Char('/'));
        a.key(Key::Char('z'));
        assert!(a.rows.is_empty());
        a.key(Key::Esc);
        assert_eq!(a.filter, "");
    }

    #[test]
    fn r_asks_for_the_files_again() {
        assert_eq!(app().key(Key::Char('r')), Action::Reload);
    }

    #[test]
    fn question_mark_opens_the_help_and_any_key_closes_it() {
        let mut a = app();
        a.key(Key::Char('?'));
        assert_eq!(a.mode, Mode::Help);
        assert_eq!(a.key(Key::Char('j')), Action::None);
        assert_eq!(a.mode, Mode::Normal);
        assert_eq!(at(&a), "Plan");
    }

    #[test]
    fn q_quits() {
        assert_eq!(app().key(Key::Char('q')), Action::Quit);
    }

    #[test]
    fn every_key_in_the_help_has_a_test() {
        // The tests above, by the keys they drive; a new key needs one.
        let tested = [
            "j k  ↓ ↑",
            "h l  ← →",
            "g G",
            "x",
            "~",
            "a A",
            "e",
            "u",
            "1 2 3",
            "/",
            "r",
            "?",
            "q",
        ];
        let keys: Vec<&str> = HELP.iter().map(|(k, _)| *k).collect();
        assert_eq!(keys, tested);
    }

    #[test]
    fn a_write_aimed_at_a_line_that_changed_is_refused() {
        let mut a = app();
        to(&mut a, "Walls");
        let Action::Write(w) = a.key(Key::Char('x')) else {
            panic!()
        };
        // An editor saved first, with a line above: Walls moved down one.
        let edited = PLAN.replace("- Build", "- Paint\n- Build");
        let e = apply(&edited, &w, today()).unwrap_err();
        assert_eq!(
            e,
            "plan.sprig:7 changed on disk since it was read; nothing was written"
        );
        // And an undo after someone else changed the file.
        let tick = a.key(Key::Char('x'));
        perform(&mut a, tick);
        let Action::Write(undo) = a.key(Key::Char('u')) else {
            panic!()
        };
        assert!(
            apply("changed", &undo, today())
                .unwrap_err()
                .contains("cannot be undone")
        );
    }

    #[test]
    fn a_reload_keeps_the_cursor_on_its_item() {
        let mut a = app();
        to(&mut a, "Walls");
        a.reload(ws(&PLAN.replace("- Design", "- Survey\n- Design")));
        assert_eq!(at(&a), "Walls");
        assert_eq!(a.rows[a.cursor].at.as_ref().unwrap().line, 8);
        // By its anchor, even when its text changes.
        to(&mut a, "Design");
        a.reload(ws(
            &PLAN.replace("- Design  ^design", "- Design work  ^design")
        ));
        assert_eq!(at(&a), "Design work");
    }
}
