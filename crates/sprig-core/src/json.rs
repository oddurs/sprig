//! Trees as JSON, in the shape `spec/tree.schema.json` describes.
//!
//! Empty lists, zero priorities and false flags are left out rather than
//! written, so a conformance example states only what matters to it; a reader
//! treats a missing key as its empty value. The schema says which keys may be
//! missing.

use serde_json::{Map, Value, json};

use crate::date::Date;
use crate::diagnostic::Diagnostic;
use crate::syntax::{Document, Node};
use crate::token::Tokens;
use crate::workspace::{Resolution, Resolved, Wait};

/// The version of the tree format. It changes only when a reader written for
/// the old version would misread the new one; adding an optional key does not
/// change it.
pub const VERSION: u64 = 1;

/// The whole document for one file of a resolution: its syntax, its resolved
/// tree, and every diagnostic in the workspace.
pub fn tree_document(res: &Resolution, file: &str, today: Date) -> Option<Value> {
    let parsed = res.parsed.get(file)?;
    let tree = res.tree(file)?;
    Some(json!({
        "sprig_tree": VERSION,
        "file": file,
        "today": today.to_string(),
        "syntax": syntax(&parsed.document),
        "resolved": resolved(&tree.root),
        "diagnostics": diagnostics(&res.diagnostics),
    }))
}

/// The syntax layer: what one file says, and nothing that depends on another
/// file or on the date.
pub fn syntax(doc: &Document) -> Value {
    json!({
        "title": doc.title,
        "root": node(&doc.root),
    })
}

fn node(n: &Node) -> Value {
    let mut m = Map::new();
    if n.line > 0 {
        m.insert("line".into(), json!(n.line));
    }
    if let Some(s) = n.span {
        m.insert("span".into(), json!([s.start, s.end]));
    }
    if let Some(mark) = n.mark {
        m.insert("mark".into(), json!(mark.as_char().to_string()));
    }
    if n.mark.is_some() || n.span.is_some() {
        m.insert("text".into(), json!(n.tokens.text));
    }
    tokens(&n.tokens, &mut m);
    if !n.notes.is_empty() {
        let notes: Vec<String> = n.notes.iter().map(|p| p.text()).collect();
        m.insert("notes".into(), json!(notes));
    }
    if !n.answers.is_empty() {
        let answers: Vec<Value> = n
            .answers
            .iter()
            .map(|a| {
                let mut m = Map::new();
                m.insert("line".into(), json!(a.line));
                m.insert("span".into(), json!([a.span.start, a.span.end]));
                m.insert("text".into(), json!(a.tokens.text));
                tokens(&a.tokens, &mut m);
                Value::Object(m)
            })
            .collect();
        m.insert("answers".into(), json!(answers));
    }
    if !n.children.is_empty() {
        let children: Vec<Value> = n.children.iter().map(node).collect();
        m.insert("children".into(), json!(children));
    }
    Value::Object(m)
}

fn tokens(t: &Tokens, m: &mut Map<String, Value>) {
    if !t.people.is_empty() {
        m.insert("people".into(), json!(t.people));
    }
    if !t.tags.is_empty() {
        m.insert("tags".into(), json!(t.tags));
    }
    if t.priority > 0 {
        m.insert("priority".into(), json!(t.priority));
    }
    if let Some(a) = t.anchor() {
        m.insert("anchor".into(), json!(a.name));
    }
    let fields = t.effective_fields();
    if !fields.is_empty() {
        let mut f = Map::new();
        for field in fields {
            f.insert(field.key.clone(), json!(field.value));
        }
        m.insert("fields".into(), Value::Object(f));
    }
    if !t.after.is_empty() {
        let after: Vec<&str> = t.after.iter().map(|a| a.value.as_str()).collect();
        m.insert("after".into(), json!(after));
    }
    if !t.links.is_empty() {
        let links: Vec<Value> = t
            .links
            .iter()
            .map(|l| match &l.anchor {
                Some(a) => json!({"path": l.path, "anchor": a}),
                None => json!({"path": l.path}),
            })
            .collect();
        m.insert("links".into(), json!(links));
    }
}

/// The resolved layer: grafts mounted, status, progress, people and blocking.
pub fn resolved(n: &Resolved) -> Value {
    let mut m = Map::new();
    m.insert("file".into(), json!(n.file));
    if n.line > 0 {
        m.insert("line".into(), json!(n.line));
    }
    if let Some(mark) = n.mark {
        m.insert("mark".into(), json!(mark.as_char().to_string()));
    }
    m.insert("label".into(), json!(n.label));
    m.insert("status".into(), json!(n.status.as_str()));
    let flag = |m: &mut Map<String, Value>, k: &str, v: bool| {
        if v {
            m.insert(k.into(), json!(true));
        }
    };
    flag(&mut m, "answered", n.answered);
    flag(&mut m, "settled", n.settled);
    flag(&mut m, "grafted", n.grafted);
    if let Some(g) = &n.graft {
        let mut gm = Map::new();
        gm.insert("path".into(), json!(g.path));
        if let Some(a) = &g.anchor {
            gm.insert("anchor".into(), json!(a));
        }
        gm.insert("title".into(), json!(g.title));
        gm.insert("mounted".into(), json!(g.mounted));
        if !g.notes.is_empty() {
            gm.insert("notes".into(), json!(g.notes));
        }
        if !g.answers.is_empty() {
            gm.insert("answers".into(), json!(g.answers));
        }
        m.insert("graft".into(), Value::Object(gm));
    }
    m.insert(
        "progress".into(),
        json!({"done": n.progress.done, "doing": n.progress.doing, "total": n.progress.total}),
    );
    if n.estimate > 0.0 {
        m.insert("estimate".into(), hours(n.estimate));
    }
    if !n.people.is_empty() {
        m.insert("people".into(), json!(n.people));
    }
    if let Some(d) = n.due {
        m.insert("due".into(), json!(d.to_string()));
    }
    if let Some(d) = n.start {
        m.insert("start".into(), json!(d.to_string()));
    }
    flag(&mut m, "blocked", n.blocked);
    if !n.waits_on.is_empty() {
        let waits: Vec<Value> = n
            .waits_on
            .iter()
            .map(|w| match w {
                Wait::Reference {
                    value,
                    label,
                    unknown,
                    target,
                } => {
                    let mut w = Map::new();
                    w.insert("after".into(), json!(value));
                    w.insert("label".into(), json!(label));
                    if *unknown {
                        w.insert("unknown".into(), json!(true));
                    }
                    if let Some((file, line)) = target {
                        w.insert("file".into(), json!(file));
                        if *line > 0 {
                            w.insert("line".into(), json!(line));
                        }
                    }
                    Value::Object(w)
                }
                Wait::Start(d) => json!({"start": d.to_string()}),
            })
            .collect();
        m.insert("waits_on".into(), json!(waits));
    }
    flag(&mut m, "ready_to_close", n.ready_to_close);
    if !n.children.is_empty() {
        let children: Vec<Value> = n.children.iter().map(resolved).collect();
        m.insert("children".into(), json!(children));
    }
    Value::Object(m)
}

// Whole hours are written as integers, so `56` and not `56.0`.
fn hours(h: f64) -> Value {
    if h.fract() == 0.0 && h.abs() < 9.0e15 {
        json!(h as i64)
    } else {
        json!(h)
    }
}

pub fn diagnostics(ds: &[Diagnostic]) -> Value {
    let out: Vec<Value> = ds
        .iter()
        .map(|d| {
            let mut m = Map::new();
            m.insert("code".into(), json!(d.code.as_str()));
            m.insert("severity".into(), json!(d.severity().as_str()));
            m.insert("file".into(), json!(d.file));
            m.insert("line".into(), json!(d.line));
            m.insert("col".into(), json!(d.col));
            m.insert("span".into(), json!([d.span.start, d.span.end]));
            m.insert("message".into(), json!(d.message));
            if let Some(h) = &d.help {
                m.insert("help".into(), json!(h));
            }
            Value::Object(m)
        })
        .collect();
    Value::Array(out)
}
