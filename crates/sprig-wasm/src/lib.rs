//! sprig-core for the website.
//!
//! The site must not carry a parser of its own (cairn 0040: one reference
//! implementation, compiled everywhere), so the playground, the landing page's
//! hero and the build-time renderer all ask this module. The interface is one
//! call that takes a JSON request and returns a JSON answer: no bindings
//! generator, and nothing the browser has to trust beyond bytes in and out.
//!
//! The answer is shaped for the site's renderer, not for interchange: that is
//! what `spec/tree.schema.json` is for. Offsets are in UTF-16 code units,
//! because that is how JavaScript indexes a string.

use std::collections::HashMap;

use serde_json::{Map, Value, json};
use sprig_core::date::{self, Date};
use sprig_core::syntax::{self, Answer, LineKind, Node, Paragraph};
use sprig_core::token::{self, Tokens};
use sprig_core::workspace::{Resolution, Resolved, Status, Wait, Workspace};
use sprig_core::{Mark, Severity, next};

/// Answers one request. Malformed requests get `{"error": ...}`, never a trap.
pub fn handle(request: &str) -> String {
    let answer = match serde_json::from_str::<Value>(request) {
        Ok(req) => dispatch(&req).unwrap_or_else(|e| json!({ "error": e })),
        Err(e) => json!({ "error": format!("not JSON: {e}") }),
    };
    answer.to_string()
}

fn dispatch(req: &Value) -> Result<Value, String> {
    let str_of = |k: &str| req.get(k).and_then(Value::as_str);
    let today = || -> Result<Date, String> {
        let t = str_of("today").ok_or("today is required")?;
        Date::parse_iso(t).ok_or_else(|| format!("today = {t} is not a date"))
    };
    match str_of("op").ok_or("op is required")? {
        "workspace" => {
            let files = req
                .get("files")
                .and_then(Value::as_object)
                .ok_or("files is required")?;
            let mut w = Workspace::new();
            for (path, src) in files {
                w.insert(path.clone(), src.as_str().ok_or("a file is a string")?);
            }
            Ok(workspace(&w.resolve(today()?)))
        }
        "lex" => Ok(lex(str_of("src").ok_or("src is required")?)),
        "date" => {
            let v = str_of("value").unwrap_or("");
            Ok(json!(date::resolve(v, today()?).map(|d| d.to_string())))
        }
        "next_due" => {
            let due = str_of("due").and_then(|d| date::resolve(d, today().ok()?));
            let every = str_of("every").ok_or("every is required")?;
            Ok(json!(
                date::next_occurrence(due, every, today()?).map(|d| d.to_string())
            ))
        }
        "expand" => {
            let line = str_of("line").ok_or("line is required")?;
            Ok(json!(syntax::expand_dates(line, today()?)))
        }
        "inline" => {
            let text = str_of("text").ok_or("text is required")?;
            let note = req.get("note").and_then(Value::as_bool).unwrap_or(false);
            Ok(inline(text, note))
        }
        "tick" => {
            let line = str_of("line").ok_or("line is required")?;
            Ok(match syntax::tick(line, today()?) {
                Some(t) => json!({"line": t.line, "movedTo": t.moved_to.map(|d| d.to_string())}),
                None => Value::Null,
            })
        }
        other => Err(format!("no such op: {other}")),
    }
}

// ----------------------------------------------------------------------------
// A workspace: every file's tree, in the shape the site renders.

fn workspace(r: &Resolution) -> Value {
    // Every line any file writes, so a grafted copy can find its notes and
    // answers, which the resolved tree does not carry.
    let mut lines: HashMap<(&str, usize), &Node> = HashMap::new();
    for (file, p) in &r.parsed {
        let root = &p.document.root;
        lines.insert((file, root.line), root);
        fn walk<'a>(file: &'a str, n: &'a Node, out: &mut HashMap<(&'a str, usize), &'a Node>) {
            for c in &n.children {
                out.insert((file, c.line), c);
                walk(file, c, out);
            }
        }
        walk(file, root, &mut lines);
    }
    let mut errors: HashMap<(&str, usize), &str> = HashMap::new();
    for d in &r.diagnostics {
        if d.severity() == Severity::Error && d.code.as_str() < "S0200" {
            errors
                .entry((d.file.as_str(), d.line))
                .or_insert(d.message.as_str());
        }
    }
    let ctx = Ctx { lines, errors };

    let mut docs = Map::new();
    for tree in r.trees() {
        let mut nodes = Vec::new();
        let root = ctx.node(&tree.root, &mut nodes);
        let mut leaves = Vec::new();
        let due = tree
            .root
            .tokens
            .field("due")
            .map(|f| (f.value.clone(), tree.root.due));
        ctx.leaves(
            &tree.root,
            &mut 0,
            &mut Vec::new(),
            None,
            due,
            0,
            &mut leaves,
        );
        // The ranking is the core's: Next up and `sprig next` agree.
        let n = next::next(&[tree], None);
        let index: HashMap<(&str, usize), usize> = leaves
            .iter()
            .enumerate()
            .map(|(i, l)| ((l.file, l.line), i))
            .collect();
        let order = |es: &[next::Entry]| -> Vec<usize> {
            es.iter()
                .filter_map(|e| index.get(&(e.node.file.as_str(), e.node.line)).copied())
                .collect()
        };
        let parsed = &r.parsed[&tree.file].document;
        docs.insert(
            tree.file.clone(),
            json!({
                "file": tree.file,
                "title": parsed.title.clone().filter(|t| !t.is_empty()),
                "root": root,
                "leaves": leaves.iter().map(Leaf::json).collect::<Vec<_>>(),
                "ready": order(&n.ready),
                "waiting": order(&n.waiting),
            }),
        );
    }
    let diagnostics: Vec<Value> = r
        .diagnostics
        .iter()
        .map(|d| {
            json!({
                "code": d.code.as_str(),
                "severity": d.severity().as_str(),
                "file": d.file,
                "line": d.line,
                "col": d.col,
                "message": d.message,
                "help": d.help,
            })
        })
        .collect();
    json!({ "docs": docs, "diagnostics": diagnostics })
}

struct Ctx<'a> {
    lines: HashMap<(&'a str, usize), &'a Node>,
    errors: HashMap<(&'a str, usize), &'a str>,
}

struct Leaf<'r> {
    id: usize,
    file: &'r str,
    line: usize,
    path: Vec<String>,
    blk: Option<Value>,
    due: Option<String>,
    due_date: Option<Date>,
    prio: u8,
}

impl Leaf<'_> {
    fn json(&self) -> Value {
        json!({
            "id": self.id,
            "path": self.path,
            "blk": self.blk,
            "due": self.due,
            "dueDate": self.due_date.map(|d| d.to_string()),
            "prio": self.prio,
        })
    }
}

fn state(n: &Resolved) -> &'static str {
    match n.mark {
        None => "root",
        Some(Mark::Todo) => "todo",
        Some(Mark::Doing) => "doing",
        Some(Mark::Done) => "done",
        Some(Mark::Dropped) => "dropped",
        Some(Mark::Later) => "later",
        Some(Mark::Question) => "ask",
        Some(Mark::Group) => "group",
        Some(Mark::Graft) => "graft",
    }
}

/// The site's name for a status: a question is `ask`.
fn view(s: Status) -> &'static str {
    match s {
        Status::Question => "ask",
        other => other.as_str(),
    }
}

/// An item's own blockers, as the renderer shows them, or `null`.
fn blockers(n: &Resolved) -> Value {
    if n.waits_on.is_empty() {
        return Value::Null;
    }
    let list: Vec<Value> = n
        .waits_on
        .iter()
        .map(|w| match w {
            Wait::Start(d) => json!({ "date": d.to_string() }),
            Wait::Reference {
                label,
                unknown,
                target,
                ..
            } => {
                let mut m = Map::new();
                m.insert("label".into(), json!(label));
                if *unknown {
                    m.insert("unknown".into(), json!(true));
                }
                if let Some((file, line)) = target {
                    m.insert("file".into(), json!(file));
                    m.insert("line".into(), json!(line.saturating_sub(1)));
                }
                Value::Object(m)
            }
            // A kind of wait a later core adds: shown without detail.
            _ => json!({ "label": "something not yet finished" }),
        })
        .collect();
    Value::Array(list)
}

fn meta(t: &Tokens) -> Value {
    let mut fields = Map::new();
    for f in t.effective_fields() {
        fields.insert(f.key.clone(), json!(f.value));
    }
    json!({
        "who": t.people,
        "tags": t.tags,
        "prio": t.priority,
        "id": t.anchor().map(|a| a.name.clone()),
        "fields": fields,
        "after": t.after.iter().map(|a| a.value.clone()).collect::<Vec<_>>(),
        "words": t.words,
        "text": t.text,
    })
}

/// Paragraphs as the renderer stores note lines: one string each, with an
/// empty string between paragraphs.
fn notes(ps: &[Paragraph]) -> Vec<String> {
    let mut out = Vec::new();
    for (i, p) in ps.iter().enumerate() {
        if i > 0 {
            out.push(String::new());
        }
        out.extend(p.lines.iter().map(|l| l.text.clone()));
    }
    out
}

fn answers(a: &[Answer]) -> Vec<Value> {
    a.iter().map(|a| meta(&a.tokens)).collect()
}

impl<'a> Ctx<'a> {
    fn node(&self, n: &Resolved, nodes: &mut Vec<()>) -> Value {
        let id = nodes.len();
        nodes.push(());
        let syn = self.lines.get(&(n.file.as_str(), n.line));
        let mut m = Map::new();
        m.insert("id".into(), json!(id));
        m.insert("state".into(), json!(state(n)));
        m.insert("view".into(), json!(view(n.status)));
        m.insert("file".into(), json!(n.file));
        // 0-based, as the site counts lines; -1 for an untitled root.
        m.insert("line".into(), json!(n.line as i64 - 1));
        m.insert("meta".into(), meta(&n.tokens));
        m.insert("label".into(), json!(n.label));
        m.insert(
            "notes".into(),
            json!(syn.map(|s| notes(&s.notes)).unwrap_or_default()),
        );
        m.insert(
            "answers".into(),
            json!(syn.map(|s| answers(&s.answers)).unwrap_or_default()),
        );
        m.insert(
            "stats".into(),
            json!({
                "total": n.progress.total,
                "done": n.progress.done,
                "doing": n.progress.doing,
                "est": n.estimate,
            }),
        );
        m.insert(
            "inh".into(),
            if n.settled {
                json!(view(n.status))
            } else {
                Value::Null
            },
        );
        m.insert("answered".into(), json!(n.answered));
        m.insert("cloned".into(), json!(n.grafted));
        m.insert("who".into(), json!(n.people));
        m.insert("blocked".into(), json!(n.blocked));
        m.insert("blk".into(), blockers(n));
        m.insert("readyToClose".into(), json!(n.ready_to_close));
        m.insert(
            "dates".into(),
            json!({
                "due": n.due.map(|d| d.to_string()),
                "start": n.start.map(|d| d.to_string()),
            }),
        );
        if let Some(g) = &n.graft {
            m.insert(
                "target".into(),
                json!((!g.path.is_empty()).then_some(&g.path)),
            );
            m.insert("targetId".into(), json!(g.anchor));
            m.insert("graftTitle".into(), json!(g.title));
            m.insert("graftOwner".into(), json!(g.owner));
            let source = g
                .source_line
                .and_then(|l| self.lines.get(&(g.path.as_str(), l)));
            m.insert(
                "graftAnswers".into(),
                json!(source.map(|s| answers(&s.answers)).unwrap_or_default()),
            );
            m.insert("graftNotes".into(), json!(notes_of(g)));
            if let Some(e) = self.errors.get(&(n.file.as_str(), n.line)) {
                m.insert("error".into(), json!(e));
            }
        }
        let children: Vec<Value> = n.children.iter().map(|c| self.node(c, nodes)).collect();
        m.insert("children".into(), Value::Array(children));
        Value::Object(m)
    }

    /// Every leaf in document order, with what Next up needs: its path, the
    /// blockers that hold it (its own, or the nearest blocked ancestor's), the
    /// nearest due date and the highest priority above it.
    #[allow(clippy::too_many_arguments)]
    fn leaves<'r>(
        &self,
        n: &'r Resolved,
        id: &mut usize,
        path: &mut Vec<String>,
        held: Option<&Value>,
        due: Option<(String, Option<Date>)>,
        prio: u8,
        out: &mut Vec<Leaf<'r>>,
    ) {
        for c in &n.children {
            *id += 1;
            let me = *id;
            let own = blockers(c);
            let blk = if c.blocked {
                if own.is_null() {
                    held.cloned()
                } else {
                    Some(own)
                }
            } else {
                None
            };
            let due = c
                .tokens
                .field("due")
                .map(|f| (f.value.clone(), c.due))
                .or_else(|| due.clone());
            let prio = prio.max(c.tokens.priority);
            if c.children.is_empty() {
                out.push(Leaf {
                    id: me,
                    file: &c.file,
                    line: c.line,
                    path: path.clone(),
                    blk,
                    due: due.as_ref().map(|d| d.0.clone()),
                    due_date: due.and_then(|d| d.1),
                    prio,
                });
            } else {
                path.push(c.label.clone());
                self.leaves(c, id, path, blk.as_ref(), due, prio, out);
                path.pop();
            }
        }
    }
}

// Graft notes come through the resolved tree as joined paragraphs.
fn notes_of(g: &sprig_core::workspace::Graft) -> Vec<String> {
    let mut out = Vec::new();
    for (i, p) in g.notes.iter().enumerate() {
        if i > 0 {
            out.push(String::new());
        }
        out.push(p.clone());
    }
    out
}

// ----------------------------------------------------------------------------
// Lexing, for the editor's colours.

fn lex(src: &str) -> Value {
    let lexed = syntax::lex(src);
    let mut starts = vec![0];
    starts.extend(src.match_indices('\n').map(|(i, _)| i + 1));
    let lines: Vec<Value> = lexed
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let line = &src[starts[i]..];
            // UTF-16 offset of a byte offset in the file, from the line start.
            let u = |b: usize| -> usize {
                line[..b.saturating_sub(starts[i]).min(line.len())]
                    .encode_utf16()
                    .count()
            };
            let kind = match l.kind {
                LineKind::Item(m) => json!(state_of_mark(m)),
                k => json!(k.as_str()),
            };
            json!({
                "kind": kind,
                "at": u(l.span.start),
                "end": u(l.span.end),
                "marker": l.marker.map(|m| [u(m.start), u(m.end)]),
                "pieces": l.pieces.iter().map(|(p, s)| json!([p.as_str(), u(s.start), u(s.end)])).collect::<Vec<_>>(),
            })
        })
        .collect();
    Value::Array(lines)
}

/// The coloured pieces of a run of text, with each link's file and anchor.
fn inline(text: &str, note: bool) -> Value {
    let u = |b: usize| text[..b].encode_utf16().count();
    let pieces: Vec<Value> = token::pieces(text, 0, note)
        .into_iter()
        .map(|(p, s)| {
            let mut m = Map::new();
            m.insert("kind".into(), json!(p.as_str()));
            m.insert("at".into(), json!(u(s.start)));
            m.insert("end".into(), json!(u(s.end)));
            if p == token::Piece::Link {
                let raw = &text[s.start..s.end];
                // A field's value may hold a link inside other text.
                if let Some((_, l)) = token::links(raw).into_iter().next() {
                    m.insert("path".into(), json!(l.path));
                    m.insert("anchor".into(), json!(l.anchor));
                }
            }
            Value::Object(m)
        })
        .collect();
    Value::Array(pieces)
}

fn state_of_mark(m: Mark) -> &'static str {
    match m {
        Mark::Todo => "todo",
        Mark::Doing => "doing",
        Mark::Done => "done",
        Mark::Dropped => "dropped",
        Mark::Later => "later",
        Mark::Question => "ask",
        Mark::Group => "group",
        Mark::Graft => "graft",
    }
}

// ----------------------------------------------------------------------------
// The WebAssembly interface: bytes in, bytes out.

/// Space for a request of `len` bytes, which the caller fills and passes to
/// [`sprig_call`].
#[unsafe(no_mangle)]
pub extern "C" fn sprig_alloc(len: usize) -> *mut u8 {
    let mut buf = vec![0u8; len].into_boxed_slice();
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Releases a buffer from [`sprig_alloc`] or an answer from [`sprig_call`].
///
/// # Safety
///
/// `ptr` and `len` must describe exactly one live buffer this module handed
/// out, and it must not be used again.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sprig_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        // SAFETY: the caller passes back a buffer this module allocated as a
        // boxed slice of exactly `len` bytes.
        drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)) });
    }
}

/// Answers the UTF-8 JSON request at `ptr`, `len`, and frees it. Returns the
/// answer's address in the high 32 bits and its length in the low 32; the
/// caller frees it with [`sprig_free`].
///
/// # Safety
///
/// `ptr` and `len` must come from [`sprig_alloc`], filled by the caller.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sprig_call(ptr: *mut u8, len: usize) -> u64 {
    // SAFETY: as for sprig_free; the request is read once and then released.
    let bytes = unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)) };
    let answer = match std::str::from_utf8(&bytes) {
        Ok(req) => handle(req),
        Err(_) => json!({ "error": "the request is not UTF-8" }).to_string(),
    };
    drop(bytes);
    let out = answer.into_bytes().into_boxed_slice();
    let (ptr, len) = (out.as_ptr() as u64, out.len() as u64);
    std::mem::forget(out);
    (ptr << 32) | len
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(req: Value) -> Value {
        serde_json::from_str(&handle(&req.to_string())).unwrap()
    }

    #[test]
    fn a_workspace_comes_back_with_trees_next_and_diagnostics() {
        let out = call(json!({
            "op": "workspace",
            "today": "2026-10-01",
            "files": {
                "a.sprig": "A  @dana\n\n- Design  ^d\n  A note.\n- Build  after:^d\n  - Strip\n+ [[b^x]]\n",
                "b.sprig": "B  @teo\n\n- X  ^x\n  = Agreed.\n  - Y  due:fri\n",
            }
        }));
        let a = &out["docs"]["a.sprig"];
        assert_eq!(a["title"], "A");
        let kids = a["root"]["children"].as_array().unwrap();
        assert_eq!(kids[0]["notes"], json!(["A note."]));
        assert_eq!(kids[1]["blk"][0]["label"], "Design");
        assert_eq!(kids[2]["state"], "graft");
        assert_eq!(kids[2]["graftTitle"], "X");
        assert_eq!(kids[2]["graftOwner"], "teo");
        assert_eq!(kids[2]["graftAnswers"][0]["text"], "Agreed.");
        assert_eq!(kids[2]["children"][0]["dates"]["due"], "2026-10-02");
        // Design and Y are ready; Strip waits on Build's blocker.
        let leaves = a["leaves"].as_array().unwrap();
        let label = |i: &Value| leaves[i.as_u64().unwrap() as usize]["path"].clone();
        assert_eq!(a["ready"].as_array().unwrap().len(), 2);
        assert_eq!(label(&a["waiting"][0]), json!(["Build"]));
        assert_eq!(leaves[1]["blk"][0]["label"], "Design");
        assert_eq!(out["diagnostics"], json!([]));
    }

    #[test]
    fn graft_errors_are_on_the_graft_node() {
        let out = call(
            json!({"op": "workspace", "today": "2026-10-01", "files": {"a.sprig": "A\n\n+ [[nowhere]]\n"}}),
        );
        let g = &out["docs"]["a.sprig"]["root"]["children"][0];
        assert_eq!(g["error"], "no file named nowhere.sprig");
        assert_eq!(out["diagnostics"][0]["code"], "S0101");
        // Where it is, for a problems list to jump to, and what to do, when
        // there is something obvious; null when there is not.
        assert_eq!(out["diagnostics"][0]["file"], "a.sprig");
        assert_eq!(out["diagnostics"][0]["line"], 3);
        assert!(out["diagnostics"][0].get("help").is_some());
    }

    #[test]
    fn lexing_counts_in_utf16_from_each_line_start() {
        let out = call(json!({"op": "lex", "src": "Plan ☃ @me\n  - Ünï  ^x"}));
        assert_eq!(out[0]["kind"], "title");
        assert_eq!(out[0]["pieces"][0], json!(["person", 7, 10]));
        assert_eq!(out[1]["kind"], "todo");
        assert_eq!(out[1]["marker"], json!([2, 3]));
        assert_eq!(out[1]["pieces"][0], json!(["anchor", 9, 11]));
    }

    #[test]
    fn dates_and_ticks_resolve_against_the_given_day() {
        assert_eq!(
            call(json!({"op": "date", "value": "fri", "today": "2026-10-01"})),
            "2026-10-02"
        );
        assert_eq!(
            call(json!({"op": "date", "value": "soon", "today": "2026-10-01"})),
            Value::Null
        );
        assert_eq!(
            call(
                json!({"op": "next_due", "due": "2026-10-02", "every": "fri", "today": "2026-10-02"})
            ),
            "2026-10-09"
        );
        assert_eq!(
            call(json!({"op": "expand", "line": "- x  due:tomorrow", "today": "2026-10-01"})),
            "- x  due:2026-10-02"
        );
        assert_eq!(
            call(
                json!({"op": "tick", "line": "  - x  every:fri  due:2026-10-02", "today": "2026-10-02"})
            ),
            json!({"line": "  - x  every:fri  due:2026-10-09", "movedTo": "2026-10-09"})
        );
    }

    #[test]
    fn inline_pieces_carry_link_targets() {
        let out = call(
            json!({"op": "inline", "text": "Ask @teo, see [[./rooms/hall^door]].", "note": true}),
        );
        assert_eq!(out[0], json!({"kind": "person", "at": 4, "end": 9}));
        assert_eq!(out[1]["path"], "rooms/hall.sprig");
        assert_eq!(out[1]["anchor"], "door");
        assert_eq!(out[1]["at"], 14);
    }

    #[test]
    fn a_bad_request_is_an_error_not_a_trap() {
        assert!(call(json!({"op": "nope"}))["error"].is_string());
        assert!(handle("{").contains("error"));
        assert!(call(json!({"op": "date"}))["error"].is_string());
    }
}
