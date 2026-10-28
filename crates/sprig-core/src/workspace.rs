//! Many files, one plan: grafts (§8), status and progress (§5), people (§6)
//! and blocking (§7).
//!
//! The caller hands over files by path; the core never reads one. Paths are
//! relative to the workspace root, the directory the caller was asked to read,
//! with `/` between components on every platform. A link that would leave that
//! root is refused (§8.2): a graft path is untrusted input.
//!
//! Every file gets its own tree, the way it reads when opened on its own. A
//! graft mounts a copy of the grafted file's tree, and the copy is then rolled
//! up again in its new place, because settling (§5.2) and people (§6.1) flow
//! down from wherever it is mounted. A reference (§7.2) asks about the file
//! that writes the anchor, in that file's own tree, so its answer does not
//! depend on who is asking.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::date::Date;
use crate::diagnostic::{Code, Diagnostic, LineIndex, Span};
use crate::syntax::{self, Answer, Mark, Node, Paragraph, Parsed};
use crate::token::{self, Tokens};

/// A set of files, by workspace-relative path.
#[derive(Clone, Debug, Default)]
pub struct Workspace {
    files: BTreeMap<String, String>,
    absent: BTreeSet<String>,
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace::default()
    }

    /// Adds a file. `path` is relative to the workspace root, uses `/`, and
    /// ends in `.sprig`.
    pub fn insert(&mut self, path: impl Into<String>, source: impl Into<String>) {
        let path = path.into();
        self.absent.remove(&path);
        self.files.insert(path, source.into());
    }

    /// Records that a file [`Workspace::wanted`] asked for does not exist, so
    /// it is not asked for again.
    pub fn mark_absent(&mut self, path: impl Into<String>) {
        self.absent.insert(path.into());
    }

    pub fn files(&self) -> impl Iterator<Item = (&str, &str)> {
        self.files.iter().map(|(p, s)| (p.as_str(), s.as_str()))
    }

    /// Files that grafts and references name but that have not been added or
    /// marked absent, inside the root. A caller that loads lazily adds these,
    /// marks the rest absent, and asks again until nothing is wanted.
    pub fn wanted(&self) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for (path, src) in &self.files {
            let parsed = syntax::parse(path, src);
            let mut want = |link: &str| {
                if let Ok(p) = join(path, link)
                    && !self.files.contains_key(&p)
                    && !self.absent.contains(&p)
                {
                    out.insert(p);
                }
            };
            syntax::walk(&parsed.document.root, &mut |n: &Node| {
                if n.mark == Some(Mark::Graft)
                    && let Some(l) = n.tokens.graft_link()
                {
                    want(&l.path);
                }
                for a in &n.tokens.after {
                    if let Some((_, l)) = token::links(&a.value).into_iter().next() {
                        want(&l.path);
                    }
                }
            });
        }
        out
    }

    /// Parses and resolves every file. `today` decides which start dates are
    /// still ahead (§7.5); nothing else depends on it.
    pub fn resolve(&self, today: Date) -> Resolution {
        let parsed: BTreeMap<String, Parsed> = self
            .files
            .iter()
            .map(|(p, s)| (p.clone(), syntax::parse(p, s)))
            .collect();
        let mut b = Builder {
            parsed: &parsed,
            index: self
                .files
                .iter()
                .map(|(p, s)| (p.as_str(), LineIndex::new(s)))
                .collect(),
            loops: HashSet::new(),
            expanded: HashMap::new(),
            rolled: HashMap::new(),
            grafted: BTreeSet::new(),
            diagnostics: Vec::new(),
        };
        b.find_loops();
        for file in parsed.keys() {
            b.rolled(file);
        }
        let lookup = Lookup::new(&parsed, &b.rolled);
        let mut diagnostics = std::mem::take(&mut b.diagnostics);
        for (file, p) in &parsed {
            diagnostics.extend(p.diagnostics.iter().cloned());
            check_references(
                file,
                &p.document.root,
                &b.index[file.as_str()],
                &lookup,
                &mut diagnostics,
            );
        }
        let grafted = std::mem::take(&mut b.grafted);
        let mut trees: BTreeMap<String, Tree> = BTreeMap::new();
        for file in parsed.keys() {
            let mut root = b.rolled[file.as_str()].clone();
            block(&mut root, false, today, &lookup);
            trees.insert(
                file.clone(),
                Tree {
                    file: file.clone(),
                    root,
                },
            );
        }
        diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        diagnostics.dedup();
        Resolution {
            parsed,
            trees,
            diagnostics,
            grafted,
        }
    }
}

/// Every file, parsed and resolved.
#[derive(Clone, Debug)]
pub struct Resolution {
    pub parsed: BTreeMap<String, Parsed>,
    trees: BTreeMap<String, Tree>,
    /// Every problem in every file, sorted by place.
    pub diagnostics: Vec<Diagnostic>,
    grafted: BTreeSet<String>,
}

impl Resolution {
    /// The file's tree, as it reads when opened on its own.
    pub fn tree(&self, file: &str) -> Option<&Tree> {
        self.trees.get(file)
    }

    pub fn trees(&self) -> impl Iterator<Item = &Tree> {
        self.trees.values()
    }

    /// Files no other file grafts whole: the tops of the plan. In path order.
    pub fn roots(&self) -> Vec<&str> {
        self.trees
            .keys()
            .filter(|f| !self.grafted.contains(*f))
            .map(String::as_str)
            .collect()
    }
}

/// One file's tree with grafts mounted and everything computed.
#[derive(Clone, Debug, PartialEq)]
pub struct Tree {
    pub file: String,
    pub root: Resolved,
}

/// Status after settling (§5.2) and derivation (§5.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    Todo,
    Doing,
    Done,
    Dropped,
    Later,
    Question,
    /// A group, graft or root with nothing beneath it that counts.
    Empty,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Todo => "todo",
            Status::Doing => "doing",
            Status::Done => "done",
            Status::Dropped => "dropped",
            Status::Later => "later",
            Status::Question => "question",
            Status::Empty => "empty",
        }
    }

    /// Open: to do, doing, or an unanswered question (§7.1).
    pub fn is_open(self) -> bool {
        matches!(self, Status::Todo | Status::Doing | Status::Question)
    }

    fn written(mark: Mark) -> Option<Status> {
        Some(match mark {
            Mark::Todo => Status::Todo,
            Mark::Doing => Status::Doing,
            Mark::Done => Status::Done,
            Mark::Dropped => Status::Dropped,
            Mark::Later => Status::Later,
            Mark::Question => Status::Question,
            Mark::Group | Mark::Graft => return None,
        })
    }

    fn settles(self) -> bool {
        matches!(self, Status::Done | Status::Dropped | Status::Later)
    }
}

/// Counting leaves beneath a node (§5.4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub done: usize,
    pub doing: usize,
    pub total: usize,
}

/// What a graft line mounted.
#[derive(Clone, Debug, PartialEq)]
pub struct Graft {
    /// The grafted file, workspace-relative. Empty when the line has no link
    /// or its path could not be resolved.
    pub path: String,
    pub anchor: Option<String>,
    /// The grafted file's title, or the anchored item's text (§8.3).
    pub title: String,
    /// The mounted branch's notes and, for a branch, its answers.
    pub notes: Vec<String>,
    pub answers: Vec<String>,
    /// False when the graft is an error and mounted nothing.
    pub mounted: bool,
}

/// Something an open item waits on (§7.3, §7.5).
#[derive(Clone, Debug, PartialEq)]
pub enum Wait {
    Reference {
        /// The `after:` value as written.
        value: String,
        /// The target's text, or the value itself when it names nothing.
        label: String,
        /// Names no existing file or anchor (§7.2).
        unknown: bool,
    },
    Start(Date),
}

/// A node of a resolved tree.
#[derive(Clone, Debug, PartialEq)]
pub struct Resolved {
    /// The file that writes this line.
    pub file: String,
    /// 1-based; 0 for the root of a file without a title.
    pub line: usize,
    /// `None` for the root.
    pub mark: Option<Mark>,
    /// The item's text; for a graft, what it mounts; for the root, the title
    /// or the file name.
    pub label: String,
    pub tokens: Tokens,
    /// Mounted here by a graft from another file.
    pub grafted: bool,
    pub graft: Option<Graft>,
    pub status: Status,
    /// A question with an answer (§2.5).
    pub answered: bool,
    /// Its status came from an `x`, `/` or `>` on it or above it (§5.2).
    pub settled: bool,
    pub progress: Progress,
    /// Remaining estimate in hours (§10.2).
    pub estimate: f64,
    pub people: Vec<String>,
    pub blocked: bool,
    /// What this item itself waits on; inherited blocking is in `blocked`.
    pub waits_on: Vec<Wait>,
    /// Written open, with every counting leaf beneath it done (§5.6).
    pub ready_to_close: bool,
    pub children: Vec<Resolved>,
}

impl Resolved {
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

// ----------------------------------------------------------------------------
// Paths

/// Resolves `link` relative to the directory of `from` (§4.8), refusing any
/// path that leaves the root (§8.2).
pub(crate) fn join(from: &str, link: &str) -> Result<String, ()> {
    if link.starts_with('/') || link.starts_with('\\') || link.contains(':') {
        return Err(());
    }
    let mut parts: Vec<&str> = from.split('/').collect();
    parts.pop();
    for part in link.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop().ok_or(())?;
            }
            p => parts.push(p),
        }
    }
    Ok(parts.join("/"))
}

fn short(path: &str) -> &str {
    path.strip_suffix(".sprig").unwrap_or(path)
}

// ----------------------------------------------------------------------------
// Expansion: a file's own lines with its grafts mounted.

#[derive(Clone)]
struct Expanded<'a> {
    file: &'a str,
    syn: &'a Node,
    grafted: bool,
    graft: Option<Mount<'a>>,
    children: Vec<Expanded<'a>>,
}

#[derive(Clone)]
struct Mount<'a> {
    path: String,
    anchor: Option<String>,
    title: String,
    owner: Option<String>,
    notes: &'a [Paragraph],
    answers: &'a [Answer],
    /// What this graft brings in, for §8.8: the file and, for a branch, the
    /// anchored line. `None` when the graft mounted nothing.
    key: Option<(String, Option<usize>)>,
    span: Span,
}

impl Expanded<'_> {
    fn label(&self) -> String {
        if let Some(g) = &self.graft {
            return g.title.clone();
        }
        if self.syn.mark.is_none() {
            return self.syn.tokens.text.clone();
        }
        let t = &self.syn.tokens.text;
        if t.is_empty() {
            "untitled".into()
        } else {
            t.clone()
        }
    }

    fn mark_grafted(&mut self) {
        self.grafted = true;
        for c in &mut self.children {
            c.mark_grafted();
        }
    }
}

struct Builder<'a> {
    parsed: &'a BTreeMap<String, Parsed>,
    index: HashMap<&'a str, LineIndex<'a>>,
    /// Graft lines, by file and line, that close a loop.
    loops: HashSet<(&'a str, usize)>,
    expanded: HashMap<&'a str, Expanded<'a>>,
    rolled: HashMap<&'a str, Resolved>,
    grafted: BTreeSet<String>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Builder<'a> {
    fn diagnose(&mut self, file: &str, code: Code, span: Span, message: String) {
        let d = self.index[file].diagnostic(file, code, span, message);
        self.diagnostics.push(d);
    }

    /// Finds every graft that would loop (§8.4), once for the whole workspace.
    ///
    /// Files are walked in path order and grafts in document order; a graft to
    /// a file still being walked closes a loop and is the one reported. Taking
    /// those grafts out leaves no loops, so every file's tree is finite, and
    /// the same graft is reported whichever file a tool happens to open first
    /// (Appendix B.10).
    fn find_loops(&mut self) {
        let mut edges: HashMap<&'a str, Vec<(usize, String)>> = HashMap::new();
        for (file, p) in self.parsed {
            let mut out = Vec::new();
            own_items(&p.document.root, &mut |n| {
                if n.mark == Some(Mark::Graft)
                    && let Some(l) = n.tokens.graft_link()
                    && let Ok(t) = join(file, &l.path)
                    && self.parsed.contains_key(&t)
                {
                    out.push((n.line, t));
                }
            });
            edges.insert(file.as_str(), out);
        }
        #[derive(PartialEq)]
        enum State {
            Walking,
            Done,
        }
        let mut state: HashMap<&str, State> = HashMap::new();
        for start in self.parsed.keys() {
            if state.contains_key(start.as_str()) {
                continue;
            }
            // An explicit stack: a long chain of grafts must not overflow ours.
            let mut stack: Vec<(&str, usize)> = vec![(start.as_str(), 0)];
            state.insert(start.as_str(), State::Walking);
            while let Some((file, next)) = stack.last_mut() {
                let out = &edges[*file];
                if *next == out.len() {
                    state.insert(file, State::Done);
                    stack.pop();
                    continue;
                }
                let (line, target) = &out[*next];
                *next += 1;
                let file: &'a str = self.parsed.get_key_value(*file).unwrap().0;
                let target: &'a str = self.parsed.get_key_value(target).unwrap().0;
                match state.get(target) {
                    Some(State::Walking) => {
                        self.loops.insert((file, *line));
                    }
                    Some(State::Done) => {}
                    None => {
                        state.insert(target, State::Walking);
                        stack.push((target, 0));
                    }
                }
            }
        }
    }

    fn expanded(&mut self, file: &'a str) -> Expanded<'a> {
        if let Some(e) = self.expanded.get(file) {
            return e.clone();
        }
        let doc = &self.parsed[file].document;
        let mut root = self.own(file, &doc.root);
        self.once_in_tree(file, &mut root);
        self.expanded.insert(file, root.clone());
        root
    }

    fn own(&mut self, file: &'a str, syn: &'a Node) -> Expanded<'a> {
        let children = syn.children.iter().map(|c| self.own(file, c)).collect();
        let mut e = Expanded {
            file,
            syn,
            grafted: false,
            graft: None,
            children,
        };
        if syn.mark == Some(Mark::Graft) {
            self.mount(&mut e);
        }
        e
    }

    /// Resolves one graft line and mounts what it names (§8.1 to §8.5).
    fn mount(&mut self, e: &mut Expanded<'a>) {
        let file = e.file;
        let syn = e.syn;
        let line_span = syn.span.expect("an item has a span");
        let mut g = Mount {
            path: String::new(),
            anchor: None,
            title: if syn.tokens.text.is_empty() {
                "graft".into()
            } else {
                syn.tokens.text.clone()
            },
            owner: None,
            notes: &[],
            answers: &[],
            key: None,
            span: line_span,
        };
        // A graft without a link was reported by the parser.
        let Some(link) = syn.tokens.graft_link() else {
            e.graft = Some(g);
            return;
        };
        g.span = link.span;
        g.anchor = link.anchor.clone();
        g.title = link.path.clone();
        let named = match &link.anchor {
            Some(a) => format!("{}^{a}", short(&link.path)),
            None => link.path.clone(),
        };
        let Ok(target) = join(file, &link.path) else {
            self.diagnose(
                file,
                Code::GraftOutsideRoot,
                link.span,
                format!("{named} is outside the directory being read, so it is not grafted"),
            );
            e.graft = Some(g);
            return;
        };
        g.path = target.clone();
        g.title = target.clone();
        if self.loops.contains(&(file, syn.line)) {
            self.diagnose(
                file,
                Code::GraftLoop,
                link.span,
                format!("{target} is already above this line, so grafting it would loop"),
            );
            e.graft = Some(g);
            return;
        }
        let Some((target, _)) = self.parsed.get_key_value(&target) else {
            self.diagnose(
                file,
                Code::GraftFileMissing,
                link.span,
                format!("no file named {target}"),
            );
            e.graft = Some(g);
            return;
        };
        let target: &'a str = target;
        let source = self.expanded(target);
        let src = match &link.anchor {
            None => &source,
            Some(a) => match find_anchor(&source, a) {
                Some(s) => s,
                None => {
                    self.diagnose(
                        file,
                        Code::GraftAnchorMissing,
                        link.span,
                        format!("no item ^{a} in {target}"),
                    );
                    e.graft = Some(g);
                    return;
                }
            },
        };
        // A file grafted whole is part of another plan; one that only lends
        // a branch is still a plan of its own.
        if target != file && link.anchor.is_none() {
            self.grafted.insert(target.to_string());
        }
        g.title = match &link.anchor {
            Some(_) => src.label(),
            None => self.parsed[target]
                .document
                .title
                .clone()
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| target.to_string()),
        };
        let rolled = self.rolled(target);
        let src_rolled = match &link.anchor {
            None => Some(rolled),
            Some(a) => find_anchor_rolled(rolled, a),
        };
        g.owner = src_rolled.and_then(|r| r.people.first().cloned());
        g.notes = &src.syn.notes;
        if link.anchor.is_some() {
            g.answers = &src.syn.answers;
        }
        g.key = Some((
            target.to_string(),
            link.anchor.as_ref().map(|_| src.syn.line),
        ));
        for c in &src.children {
            let mut c = c.clone();
            c.mark_grafted();
            e.children.push(c);
        }
        e.graft = Some(g);
    }

    /// §8.8: walking a file's own lines in document order, a graft that would
    /// bring in anything already in the tree mounts nothing.
    fn once_in_tree(&mut self, file: &'a str, root: &mut Expanded<'a>) {
        let mut seen: HashSet<(String, Option<usize>)> = HashSet::new();
        let mut errors = Vec::new();
        once_walk(root, &mut seen, &mut errors);
        for (span, message) in errors {
            self.diagnose(file, Code::GraftRepeated, span, message);
        }
    }

    fn rolled(&mut self, file: &'a str) -> &Resolved {
        if !self.rolled.contains_key(file) {
            let e = self.expanded(file);
            let people = e.syn.tokens.people.clone();
            let r = roll(&e, None, &people);
            self.rolled.insert(file, r);
        }
        &self.rolled[file]
    }
}

fn once_walk(
    n: &mut Expanded<'_>,
    seen: &mut HashSet<(String, Option<usize>)>,
    errors: &mut Vec<(Span, String)>,
) {
    for c in &mut n.children {
        if c.grafted {
            continue;
        }
        seen.insert((c.file.to_string(), Some(c.syn.line)));
        if let Some(g) = &mut c.graft
            && let Some(key) = g.key.clone()
        {
            let mut incoming = vec![key.clone()];
            for k in c.children.iter().filter(|k| k.grafted) {
                within(k, &mut incoming);
            }
            if incoming.iter().any(|k| seen.contains(k)) {
                let name = match &g.anchor {
                    Some(a) => format!("{}^{a}", short(&key.0)),
                    None => key.0.clone(),
                };
                errors.push((
                    g.span,
                    format!(
                        "{name} is already in this tree, so grafting it again would count it twice"
                    ),
                ));
                g.key = None;
                g.notes = &[];
                g.answers = &[];
                c.children.retain(|k| !k.grafted);
            } else {
                seen.extend(incoming);
            }
        }
        once_walk(c, seen, errors);
    }
}

fn within(n: &Expanded<'_>, out: &mut Vec<(String, Option<usize>)>) {
    out.push((n.file.to_string(), Some(n.syn.line)));
    if let Some(k) = n.graft.as_ref().and_then(|g| g.key.clone()) {
        out.push(k);
    }
    for c in &n.children {
        within(c, out);
    }
}

/// Visits a file's own items in document order: everything but grafted copies.
fn own_items<'a>(n: &'a Node, f: &mut impl FnMut(&'a Node)) {
    syntax::walk(n, f);
}

/// The first item the file writes itself with this anchor (§8.7, Appendix
/// B.4).
fn find_anchor<'e, 'a>(n: &'e Expanded<'a>, anchor: &str) -> Option<&'e Expanded<'a>> {
    for c in n.children.iter().filter(|c| !c.grafted) {
        if c.syn.tokens.anchor().is_some_and(|a| a.name == anchor) {
            return Some(c);
        }
        if let Some(hit) = find_anchor(c, anchor) {
            return Some(hit);
        }
    }
    None
}

fn find_anchor_rolled<'r>(n: &'r Resolved, anchor: &str) -> Option<&'r Resolved> {
    for c in n.children.iter().filter(|c| !c.grafted) {
        if c.tokens.anchor().is_some_and(|a| a.name == anchor) {
            return Some(c);
        }
        if let Some(hit) = find_anchor_rolled(c, anchor) {
            return Some(hit);
        }
    }
    None
}

// ----------------------------------------------------------------------------
// Rolling up: status, progress, estimates and people.

fn roll(e: &Expanded<'_>, inherited: Option<Status>, people: &[String]) -> Resolved {
    let syn = e.syn;
    let answered = syn.mark == Some(Mark::Question) && !syn.answers.is_empty();
    let written = syn.mark.and_then(Status::written);
    let own = if answered {
        Some(Status::Done)
    } else {
        written
    };
    let settled = inherited.or(own.filter(|s| s.settles()));
    let people: Vec<String> = if !syn.tokens.people.is_empty() {
        syn.tokens.people.clone()
    } else if let Some(owner) = e.graft.as_ref().and_then(|g| g.owner.clone()) {
        vec![owner]
    } else {
        people.to_vec()
    };
    let own_estimate = syn
        .tokens
        .field("est")
        .and_then(|f| syntax::estimate_hours(&f.value));

    let mut progress = Progress::default();
    let mut estimate = 0.0;
    let children: Vec<Resolved> = e
        .children
        .iter()
        .map(|c| roll(c, settled, &people))
        .collect();
    if children.is_empty() {
        // §5.3: a leaf counts unless it is dropped, parked, structure, or upkeep.
        let status = settled.or(own);
        let upkeep = syn.tokens.field("every").is_some();
        if let Some(s) = status
            && !upkeep
            && matches!(
                s,
                Status::Todo | Status::Doing | Status::Done | Status::Question
            )
        {
            progress.total = 1;
            progress.done = usize::from(s == Status::Done);
            progress.doing = usize::from(s == Status::Doing);
            if s != Status::Done {
                estimate = own_estimate.unwrap_or(0.0);
            }
        }
    } else {
        for c in &children {
            progress.total += c.progress.total;
            progress.done += c.progress.done;
            progress.doing += c.progress.doing;
            estimate += c.estimate;
        }
        // §10.2: a parent's own estimate stands in only when nothing beneath
        // it contributes one.
        if estimate == 0.0
            && settled.is_none()
            && let Some(h) = own_estimate
        {
            estimate = h;
        }
    }

    let derived = if progress.total == 0 {
        Status::Empty
    } else if progress.done == progress.total {
        Status::Done
    } else if progress.doing > 0 || progress.done > 0 {
        Status::Doing
    } else {
        Status::Todo
    };
    let status = settled.unwrap_or(match own {
        Some(s) if syn.mark.is_some_and(|m| !m.is_structure()) => s,
        _ => derived,
    });
    let ready_to_close = settled.is_none()
        && written.is_some_and(Status::is_open)
        && !answered
        && !children.is_empty()
        && progress.total > 0
        && progress.done == progress.total;

    let graft = e.graft.as_ref().map(|g| Graft {
        path: g.path.clone(),
        anchor: g.anchor.clone(),
        title: g.title.clone(),
        notes: g.notes.iter().map(Paragraph::text).collect(),
        answers: g.answers.iter().map(|a| a.tokens.text.clone()).collect(),
        mounted: g.key.is_some(),
    });
    let label = if syn.mark.is_none() && syn.tokens.text.is_empty() {
        e.file.to_string()
    } else {
        e.label()
    };
    Resolved {
        file: e.file.to_string(),
        line: syn.line,
        mark: syn.mark,
        label,
        tokens: syn.tokens.clone(),
        grafted: e.grafted,
        graft,
        status,
        answered,
        settled: settled.is_some(),
        progress,
        estimate,
        people,
        blocked: false,
        waits_on: Vec::new(),
        ready_to_close,
        children,
    }
}

// ----------------------------------------------------------------------------
// References and blocking.

struct Target {
    done: bool,
    label: String,
}

/// What every anchor and every file resolves to, in its own file's tree.
struct Lookup<'a> {
    parsed: &'a BTreeMap<String, Parsed>,
    anchors: HashMap<(String, String), Target>,
    files: HashMap<String, Target>,
}

enum Outcome {
    Found(bool, String),
    Unknown(Code, String),
}

impl<'a> Lookup<'a> {
    fn new(parsed: &'a BTreeMap<String, Parsed>, rolled: &HashMap<&str, Resolved>) -> Lookup<'a> {
        let mut anchors = HashMap::new();
        let mut files = HashMap::new();
        for (file, root) in rolled {
            files.insert(
                file.to_string(),
                Target {
                    done: root.status == Status::Done,
                    label: root.label.clone(),
                },
            );
            fn walk(n: &Resolved, file: &str, out: &mut HashMap<(String, String), Target>) {
                for c in n.children.iter().filter(|c| !c.grafted) {
                    if let Some(a) = c.tokens.anchor() {
                        out.entry((file.to_string(), a.name.clone()))
                            .or_insert_with(|| Target {
                                done: c.status == Status::Done,
                                label: c.label.clone(),
                            });
                    }
                    walk(c, file, out);
                }
            }
            walk(root, file, &mut anchors);
        }
        Lookup {
            parsed,
            anchors,
            files,
        }
    }

    fn resolve(&self, from: &str, value: &str) -> Outcome {
        if let Some(name) = value.strip_prefix('^')
            && !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return match self.anchors.get(&(from.to_string(), name.to_string())) {
                Some(t) => Outcome::Found(t.done, t.label.clone()),
                None => Outcome::Unknown(
                    Code::UnknownLocalAnchor,
                    format!("no item ^{name} in {from}"),
                ),
            };
        }
        let Some((_, link)) = token::links(value).into_iter().next() else {
            return Outcome::Unknown(
                Code::NotAReference,
                format!("after:{value} is not a reference, so this item waits on it forever"),
            );
        };
        let Ok(path) = join(from, &link.path) else {
            return Outcome::Unknown(
                Code::ReferenceOutsideRoot,
                format!("{} is outside the directory being read", link.path),
            );
        };
        if !self.parsed.contains_key(&path) {
            return Outcome::Unknown(Code::UnknownFile, format!("no file named {path}"));
        }
        match &link.anchor {
            Some(a) => match self.anchors.get(&(path.clone(), a.clone())) {
                Some(t) => Outcome::Found(t.done, t.label.clone()),
                None => Outcome::Unknown(Code::UnknownAnchor, format!("no item ^{a} in {path}")),
            },
            None => {
                let t = &self.files[&path];
                Outcome::Found(t.done, t.label.clone())
            }
        }
    }
}

/// Reports every `after:` that names nothing, once, on the line that writes it.
fn check_references(
    file: &str,
    root: &Node,
    index: &LineIndex,
    lookup: &Lookup,
    out: &mut Vec<Diagnostic>,
) {
    let mut check = |tokens: &Tokens| {
        for a in &tokens.after {
            if let Outcome::Unknown(code, message) = lookup.resolve(file, &a.value) {
                let mut d = index.diagnostic(file, code, a.span, message);
                if code == Code::NotAReference {
                    d.help =
                        Some("write after:^anchor, after:[[file]] or after:[[file^anchor]]".into());
                }
                out.push(d);
            }
        }
    };
    check(&root.tokens);
    syntax::walk(root, &mut |n: &Node| {
        check(&n.tokens);
        for a in &n.answers {
            check(&a.tokens);
        }
    });
}

/// §7.3: an open item is blocked by its own unfinished references and future
/// start date, and by any blocked ancestor.
fn block(n: &mut Resolved, inherited: bool, today: Date, lookup: &Lookup) {
    for c in &mut n.children {
        let open = c.status.is_open();
        let mut waits = Vec::new();
        if open {
            if let Some(start) = c
                .tokens
                .field("start")
                .and_then(|f| Date::parse_iso(&f.value))
                && start > today
            {
                waits.push(Wait::Start(start));
            }
            for a in &c.tokens.after {
                match lookup.resolve(&c.file, &a.value) {
                    Outcome::Found(true, _) => {}
                    Outcome::Found(false, label) => waits.push(Wait::Reference {
                        value: a.value.clone(),
                        label,
                        unknown: false,
                    }),
                    Outcome::Unknown(..) => waits.push(Wait::Reference {
                        value: a.value.clone(),
                        label: a.value.clone(),
                        unknown: true,
                    }),
                }
            }
        }
        c.blocked = open && (inherited || !waits.is_empty());
        c.waits_on = waits;
        let blocked = c.blocked;
        block(c, blocked, today, lookup);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> Date {
        Date::parse_iso("2026-10-01").unwrap()
    }

    fn ws(files: &[(&str, &str)]) -> Resolution {
        let mut w = Workspace::new();
        for (p, s) in files {
            w.insert(*p, *s);
        }
        w.resolve(today())
    }

    fn codes(r: &Resolution) -> Vec<(&str, usize, &str)> {
        r.diagnostics
            .iter()
            .map(|d| (d.file.as_str(), d.line, d.code.as_str()))
            .collect()
    }

    fn find<'r>(n: &'r Resolved, label: &str) -> &'r Resolved {
        fn go<'r>(n: &'r Resolved, label: &str) -> Option<&'r Resolved> {
            if n.label == label {
                return Some(n);
            }
            n.children.iter().find_map(|c| go(c, label))
        }
        go(n, label).unwrap_or_else(|| panic!("no node {label}"))
    }

    fn labels(n: &Resolved) -> Vec<&str> {
        n.children.iter().map(|c| c.label.as_str()).collect()
    }

    #[test]
    fn paths_resolve_relative_to_the_linking_file() {
        assert_eq!(join("a.sprig", "b.sprig"), Ok("b.sprig".into()));
        assert_eq!(join("rooms/a.sprig", "b.sprig"), Ok("rooms/b.sprig".into()));
        assert_eq!(join("rooms/a.sprig", "../b.sprig"), Ok("b.sprig".into()));
        assert_eq!(
            join("a.sprig", "rooms/./hall.sprig"),
            Ok("rooms/hall.sprig".into())
        );
    }

    #[test]
    fn paths_that_leave_the_root_are_refused() {
        for bad in [
            "../b.sprig",
            "../../etc/passwd.sprig",
            "/etc/passwd.sprig",
            "C:/x.sprig",
            "\\\\host\\x.sprig",
        ] {
            assert_eq!(join("a.sprig", bad), Err(()), "{bad}");
        }
        assert_eq!(join("rooms/a.sprig", "../../b.sprig"), Err(()));
    }

    #[test]
    fn a_whole_file_graft_mounts_its_top_level_items() {
        let r = ws(&[
            ("house.sprig", "House  @sam\n\n+ [[kitchen]]\n- Insurance\n"),
            ("kitchen.sprig", "Kitchen\n\n- Oven\n  - Quote\n- Sink\n"),
        ]);
        assert!(r.diagnostics.is_empty(), "{:?}", r.diagnostics);
        let root = &r.tree("house.sprig").unwrap().root;
        assert_eq!(labels(root), ["Kitchen", "Insurance"]);
        let g = &root.children[0];
        assert_eq!(labels(g), ["Oven", "Sink"]);
        assert!(
            g.children
                .iter()
                .all(|c| c.grafted && c.file == "kitchen.sprig")
        );
        assert_eq!(g.people, ["sam"]);
        assert_eq!(r.roots(), ["house.sprig"]);
    }

    #[test]
    fn a_branch_graft_mounts_exactly_that_branch_and_its_notes() {
        let r = ws(&[
            ("dana.sprig", "Dana  @dana\n\n+ [[kitchen^oven]]\n"),
            (
                "kitchen.sprig",
                "Kitchen  @teo\n\n- Deck oven  ^oven\n  Mono is cheaper.\n  = Buy the Mono.\n  - Quote\n  - Order\n- Sink\n",
            ),
        ]);
        assert!(r.diagnostics.is_empty(), "{:?}", r.diagnostics);
        let g = &r.tree("dana.sprig").unwrap().root.children[0];
        assert_eq!(g.label, "Deck oven");
        assert_eq!(labels(g), ["Quote", "Order"]);
        let graft = g.graft.as_ref().unwrap();
        assert_eq!(graft.notes, ["Mono is cheaper."]);
        assert_eq!(graft.answers, ["Buy the Mono."]);
        // §6.1: a graft that names nobody takes the branch's first person.
        assert_eq!(g.people, ["teo"]);
    }

    #[test]
    fn a_graft_to_a_missing_file_or_anchor_is_reported_on_the_graft_line() {
        let r = ws(&[
            ("a.sprig", "A\n\n+ [[nowhere]]\n+ [[b^nope]]\n"),
            ("b.sprig", "B\n\n- one\n"),
        ]);
        assert_eq!(
            codes(&r),
            [("a.sprig", 3, "S0101"), ("a.sprig", 4, "S0102")]
        );
        assert_eq!(r.diagnostics[0].col, 3);
        assert!(
            r.tree("a.sprig")
                .unwrap()
                .root
                .children
                .iter()
                .all(|g| g.children.is_empty())
        );
    }

    #[test]
    fn a_three_file_loop_is_one_error_on_the_closing_graft() {
        let r = ws(&[
            ("a.sprig", "A\n\n+ [[b]]\n"),
            ("b.sprig", "B\n\n+ [[c]]\n"),
            ("c.sprig", "C\n\n- work\n+ [[a]]\n"),
        ]);
        assert_eq!(codes(&r), [("c.sprig", 4, "S0103")]);
        let a = &r.tree("a.sprig").unwrap().root;
        assert_eq!(a.progress.total, 1);
    }

    #[test]
    fn a_long_chain_of_grafts_does_not_overflow_the_stack() {
        let n = 400;
        let mut w = Workspace::new();
        for i in 0..n {
            w.insert(
                format!("f{i}.sprig"),
                format!("F{i}\n\n- work\n+ [[f{}]]\n", (i + 1) % n),
            );
        }
        // Resolving builds every tree; deep ones are fine on a big stack, and
        // the loop is still found exactly once.
        let r = std::thread::Builder::new()
            .stack_size(256 << 20)
            .spawn(move || w.resolve(today()))
            .unwrap()
            .join()
            .unwrap();
        let loops: Vec<_> = r
            .diagnostics
            .iter()
            .filter(|d| d.code == Code::GraftLoop)
            .collect();
        assert_eq!(loops.len(), 1);
    }

    #[test]
    fn a_graft_that_leaves_the_workspace_is_refused() {
        let r = ws(&[("a.sprig", "A\n\n+ [[../../etc/passwd]]\n- after:[[../x]]\n")]);
        assert_eq!(
            codes(&r),
            [("a.sprig", 3, "S0104"), ("a.sprig", 4, "S0204")]
        );
        let mut w = Workspace::new();
        w.insert("a.sprig", "A\n\n+ [[../../etc/passwd]]\n");
        assert!(
            w.wanted().is_empty(),
            "nothing outside the root is ever asked for"
        );
    }

    #[test]
    fn grafted_content_appears_once() {
        let r = ws(&[
            ("a.sprig", "A\n\n+ [[kitchen]]\n+ [[kitchen^oven]]\n"),
            ("kitchen.sprig", "K\n\n- Oven  ^oven\n  - Quote\n"),
        ]);
        assert_eq!(codes(&r), [("a.sprig", 4, "S0105")]);
        assert_eq!(r.tree("a.sprig").unwrap().root.progress.total, 1);
    }

    #[test]
    fn settling_rolls_down_and_progress_rolls_up() {
        let r = ws(&[(
            "p.sprig",
            "P\n\nx Kitchen\n  - Tiling\n  - Paint\n- Hall\n  x Strip\n  ~ Tile\n  - Paint\n/ Movers\n> Repaint\n- Water  every:sat\n",
        )]);
        let root = &r.tree("p.sprig").unwrap().root;
        let k = find(root, "Kitchen");
        assert_eq!(
            k.progress,
            Progress {
                done: 2,
                doing: 0,
                total: 2
            }
        );
        assert!(
            k.children
                .iter()
                .all(|c| c.status == Status::Done && c.settled)
        );
        let h = find(root, "Hall");
        assert_eq!(
            h.progress,
            Progress {
                done: 1,
                doing: 1,
                total: 3
            }
        );
        assert_eq!(h.status, Status::Todo, "a parent keeps its written status");
        assert_eq!(
            root.progress,
            Progress {
                done: 3,
                doing: 1,
                total: 5
            }
        );
        assert_eq!(root.status, Status::Doing);
    }

    #[test]
    fn groups_derive_their_status() {
        let r = ws(&[(
            "p.sprig",
            "P\n\n# Dairy\n  x Butter\n  x Milk\n# Seeds\n  - Sesame\n# Empty\n# Dropped\n  / x\n",
        )]);
        let root = &r.tree("p.sprig").unwrap().root;
        let st: Vec<_> = root.children.iter().map(|c| c.status).collect();
        assert_eq!(
            st,
            [Status::Done, Status::Todo, Status::Empty, Status::Empty]
        );
    }

    #[test]
    fn an_answered_question_is_done_and_settles_its_branch() {
        let r = ws(&[(
            "p.sprig",
            "P\n\n? Tier?  ^tier\n  = Yes.\n  - Ask legal\n? Open?\n",
        )]);
        let root = &r.tree("p.sprig").unwrap().root;
        let q = &root.children[0];
        assert!(q.answered);
        assert_eq!(q.status, Status::Done);
        assert_eq!(q.children[0].status, Status::Done);
        assert_eq!(root.children[1].status, Status::Question);
        assert_eq!(
            root.progress,
            Progress {
                done: 1,
                doing: 0,
                total: 2
            }
        );
    }

    #[test]
    fn a_parent_with_every_child_done_is_ready_to_close() {
        let r = ws(&[(
            "p.sprig",
            "P\n\n- Design\n  x Layout\n  x Plan\n- Build\n  x One\n  - Two\n",
        )]);
        let root = &r.tree("p.sprig").unwrap().root;
        let d = &root.children[0];
        assert_eq!(d.status, Status::Todo);
        assert!(d.ready_to_close);
        assert!(!root.children[1].ready_to_close);
    }

    #[test]
    fn estimates_sum_open_counting_leaves() {
        let r = ws(&[(
            "p.sprig",
            "P\n\n- Kitchen  est:10d\n  x Strip  est:3d\n  - Tiling  est:5d\n  - Paint  est:2d\n- Hall  est:4h\n  - Paint\nx Done  est:3w\n  - a\n",
        )]);
        let root = &r.tree("p.sprig").unwrap().root;
        assert_eq!(find(root, "Kitchen").estimate, 56.0);
        assert_eq!(
            find(root, "Hall").estimate,
            4.0,
            "the parent's own estimate stands in"
        );
        assert_eq!(
            find(root, "Done").estimate,
            0.0,
            "a settled parent's estimate is not used"
        );
        assert_eq!(root.estimate, 60.0);
    }

    #[test]
    fn people_flow_down_until_someone_is_named() {
        let r = ws(&[(
            "p.sprig",
            "Owners  @dana\n\n- Money\n  - Loan\n  - Credit  @teo\n    - Call\n",
        )]);
        let root = &r.tree("p.sprig").unwrap().root;
        assert_eq!(find(root, "Loan").people, ["dana"]);
        assert_eq!(find(root, "Call").people, ["teo"]);
    }

    #[test]
    fn local_cross_file_and_whole_file_references_block() {
        let r = ws(&[
            (
                "a.sprig",
                "A\n\n- Design  ^design\n- Build  after:^design\n- Fit  after:[[b^signed]]\n- Open  after:[[b]]\n- Paint  after:[[c^done]]\n",
            ),
            ("b.sprig", "B\n\n- Sign  ^signed\n"),
            ("c.sprig", "C\n\nx Done  ^done\n"),
        ]);
        assert!(r.diagnostics.is_empty(), "{:?}", r.diagnostics);
        let root = &r.tree("a.sprig").unwrap().root;
        let blocked = |l| find(root, l).blocked;
        assert!(!blocked("Design"));
        assert!(blocked("Build"));
        assert!(blocked("Fit"));
        assert!(blocked("Open"));
        assert!(!blocked("Paint"));
        assert_eq!(
            find(root, "Fit").waits_on,
            [Wait::Reference {
                value: "[[b^signed]]".into(),
                label: "Sign".into(),
                unknown: false
            }]
        );
    }

    #[test]
    fn blocking_inherited_from_an_ancestor_reaches_every_open_descendant() {
        let r = ws(&[(
            "p.sprig",
            "P\n\n- Design  ^design\n- Build  after:^design\n  - Strip\n  x Done already\n  - Tiling\n    - Grout\n",
        )]);
        let root = &r.tree("p.sprig").unwrap().root;
        for l in ["Build", "Strip", "Tiling", "Grout"] {
            assert!(find(root, l).blocked, "{l}");
        }
        assert!(!find(root, "Done already").blocked);
        assert!(
            find(root, "Strip").waits_on.is_empty(),
            "inherited, not its own"
        );
    }

    #[test]
    fn an_unknown_reference_is_reported_and_blocks() {
        let r = ws(&[(
            "p.sprig",
            "P\n\n- Sign  after:^permt\n- Ship  after:[[nowhere]]\n- Go  after:tomorrow\n",
        )]);
        assert_eq!(
            codes(&r),
            [
                ("p.sprig", 3, "S0201"),
                ("p.sprig", 4, "S0202"),
                ("p.sprig", 5, "S0205")
            ]
        );
        let root = &r.tree("p.sprig").unwrap().root;
        assert!(root.children.iter().all(|c| c.blocked));
    }

    #[test]
    fn a_future_start_date_blocks_until_that_day() {
        let r = ws(&[(
            "p.sprig",
            "P\n\n- Van  start:2026-11-02\n- Tyres  start:2026-11-15\n  - Garage\n- Today  start:2026-10-01\n",
        )]);
        let root = &r.tree("p.sprig").unwrap().root;
        assert!(find(root, "Van").blocked);
        assert!(find(root, "Garage").blocked);
        assert!(!find(root, "Today").blocked);
    }

    #[test]
    fn references_reach_only_anchors_the_file_writes_itself() {
        let r = ws(&[
            ("a.sprig", "A\n\n+ [[b]]\n- Go  after:^inner\n"),
            ("b.sprig", "B\n\n- Inner  ^inner\n"),
        ]);
        assert_eq!(codes(&r), [("a.sprig", 4, "S0201")]);
    }

    #[test]
    fn wanted_names_files_not_yet_loaded() {
        let mut w = Workspace::new();
        w.insert("a.sprig", "A\n\n+ [[rooms/b]]\n- x  after:[[c^d]]\n");
        assert_eq!(
            w.wanted().into_iter().collect::<Vec<_>>(),
            ["c.sprig", "rooms/b.sprig"]
        );
        w.mark_absent("c.sprig");
        w.insert("rooms/b.sprig", "B\n");
        assert!(w.wanted().is_empty());
    }
}
