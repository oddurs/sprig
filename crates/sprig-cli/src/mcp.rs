//! `sprig mcp`: a folder's plans for a coding agent, over the Model Context
//! Protocol (spec/tools.md T6).
//!
//! JSON-RPC 2.0 on standard input and output, one message per line, as the
//! protocol's stdio transport specifies. Standard output carries the protocol
//! and nothing else. A tool that fails says so in its result, with
//! `isError`, so the model can read what went wrong and try again; only a
//! malformed request or an unknown tool is a protocol error.
//!
//! Every call reads the files afresh: a person, an editor and the agent can
//! all change them between calls, and the file is the truth. Reads return
//! what the matching `--json` command prints, built by the same functions.
//! Writes go through the core's edit API, are refused when they would add an
//! error `sprig check` reports, and return the diff they applied.

use std::collections::{BTreeMap, HashMap};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use similar::TextDiff;
use sprig_core::edit::{self, Change, Place};
use sprig_core::workspace::Resolution;
use sprig_core::{Date, Diagnostic, Severity, Workspace, json as tree, query, refs};

use crate::args::parse_mark;
use crate::data::{check_json, find_json, next_json, status_json};
use crate::load::{self, Loaded};
use crate::verbs::{appended, freed, item_line};
use crate::write::save;

/// The newest protocol revision this server speaks; it answers a client that
/// asks for an older one it also speaks in that one.
const PROTOCOL: &str = "2025-11-25";
const OLDER: &[&str] = &["2025-06-18", "2025-03-26"];

const INSTRUCTIONS: &str = "This folder's plans are Sprig files (.sprig): plain text, one \
item per line, a mark first (- to do, ~ doing, x done, / dropped, > later, ? question). \
Read PLAN.sprig, or call next_items, before starting work. Plan before changing code: add \
the steps with add_items, mark the one you start with mark_item (~), and tick_items when it \
is done. Ask a person by adding a ? item that names them (@name); do not answer your own \
questions. Every write returns the diff it made and what it unblocked, and a write that \
would break the plan is refused with the reason.";

pub struct Server<'a> {
    root: PathBuf,
    today: &'a dyn Fn() -> Date,
    /// How many items `add_items` may add, in one call and in the session.
    max_new: usize,
    added: usize,
}

/// Runs the server on standard input and output until input ends.
pub fn stdio(root: PathBuf, today: &dyn Fn() -> Date, max_new: usize) -> crate::Result<()> {
    let mut server = Server {
        root,
        today,
        max_new,
        added: 0,
    };
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(reply) = server.handle(&line) {
            writeln!(stdout, "{reply}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

impl Server<'_> {
    /// The reply to one message, or `None` for a notification.
    pub fn handle(&mut self, line: &str) -> Option<Value> {
        let msg: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => return Some(error(Value::Null, -32700, &format!("parse error: {e}"))),
        };
        // A notification has no id and takes no reply.
        let id = msg.get("id").cloned()?;
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));
        Some(match method {
            "initialize" => {
                let asked = params["protocolVersion"].as_str().unwrap_or(PROTOCOL);
                let version = if OLDER.contains(&asked) {
                    asked
                } else {
                    PROTOCOL
                };
                ok(
                    id,
                    json!({
                        "protocolVersion": version,
                        "capabilities": { "tools": { "listChanged": false } },
                        "serverInfo": {
                            "name": "sprig",
                            "title": "Sprig plans",
                            "version": env!("CARGO_PKG_VERSION"),
                        },
                        "instructions": INSTRUCTIONS,
                    }),
                )
            }
            "ping" => ok(id, json!({})),
            "tools/list" => ok(id, json!({ "tools": tools() })),
            "tools/call" => {
                let name = params["name"].as_str().unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(json!({}));
                if !tools().iter().any(|t| t["name"] == name) {
                    return Some(error(id, -32602, &format!("unknown tool: {name}")));
                }
                ok(id, self.call(name, &args))
            }
            other => error(id, -32601, &format!("unknown method: {other}")),
        })
    }

    /// A tool's result: the JSON as text, as the matching command prints it,
    /// and as structured content, which must be an object.
    pub fn call(&mut self, name: &str, args: &Value) -> Value {
        match self.dispatch(name, args) {
            Ok(v) => {
                let structured = if v.is_object() {
                    v.clone()
                } else {
                    json!({ "items": v })
                };
                json!({
                    "content": [{ "type": "text", "text": pretty(&v) }],
                    "structuredContent": structured,
                    "isError": false,
                })
            }
            Err(e) => json!({ "content": [{ "type": "text", "text": e }], "isError": true }),
        }
    }

    fn dispatch(&mut self, name: &str, a: &Value) -> Result<Value, String> {
        let today = (self.today)();
        let g = self.load()?;
        let res = g.workspace.resolve(today);
        let groups = std::slice::from_ref(&g);
        let resolved = std::slice::from_ref(&res);
        match name {
            "list_plans" => Ok(status_json(groups, resolved, today)),
            "show_tree" => {
                let file = string(a, "file")?;
                let file = if file.ends_with(".sprig") {
                    file.to_string()
                } else {
                    format!("{file}.sprig")
                };
                let key = file.strip_prefix("./").unwrap_or(&file).replace('\\', "/");
                tree::tree_document(&res, &key, today).ok_or_else(|| {
                    format!("there is no plan {file} in this folder; list_plans names them")
                })
            }
            "next_items" => {
                let who = a["who"].as_str().map(|w| w.strip_prefix('@').unwrap_or(w));
                let limit = a["limit"].as_u64().map(|n| n as usize);
                Ok(next_json(groups, resolved, who, limit))
            }
            "find_items" => {
                let q = query::parse(string(a, "query")?, today).map_err(|e| e.to_string())?;
                Ok(find_json(resolved, &q))
            }
            "check_plans" => Ok(check_json(groups, resolved)),
            "tick_items" => {
                let refs = strings(a, "refs")?;
                if refs.is_empty() {
                    return Err("refs names nothing to tick".into());
                }
                let mut texts: BTreeMap<String, String> = BTreeMap::new();
                let mut changed = Vec::new();
                for r in refs {
                    let item = refs::resolve(&res, &r).map_err(|e| e.to_string())?;
                    let src = texts
                        .entry(item.file.clone())
                        .or_insert_with(|| source(&g, &item.file).to_string());
                    let (change, moved) =
                        edit::tick(src, item.line, today).map_err(|e| edit_error(&item, e))?;
                    *src = change.source;
                    changed.push(json!({
                        "file": g.display(&item.file),
                        "line": item.line,
                        "text": line_of(src, item.line),
                        "due_moved_to": moved.map(|d| d.to_string()),
                    }));
                }
                // Ticks change no line's number.
                self.commit(
                    &g,
                    &res,
                    texts,
                    |_, l| l,
                    today,
                    json!({ "ticked": changed }),
                )
            }
            "mark_item" => {
                let item = refs::resolve(&res, string(a, "ref")?).map_err(|e| e.to_string())?;
                let mark = parse_mark(string(a, "mark")?)?.0;
                let change = edit::set_mark(source(&g, &item.file), item.line, mark)
                    .map_err(|e| edit_error(&item, e))?;
                let text = line_of(&change.source, item.line);
                let report =
                    json!({ "file": g.display(&item.file), "line": item.line, "text": text });
                self.commit(&g, &res, one(&item.file, change), |_, l| l, today, report)
            }
            "set_field" => {
                let item = refs::resolve(&res, string(a, "ref")?).map_err(|e| e.to_string())?;
                let key = string(a, "key")?;
                let value = a["value"].as_str().filter(|v| !v.is_empty());
                let change = edit::set_field(source(&g, &item.file), item.line, key, value)
                    .map_err(|e| edit_error(&item, e))?;
                let text = line_of(&change.source, item.line);
                let report =
                    json!({ "file": g.display(&item.file), "line": item.line, "text": text });
                self.commit(&g, &res, one(&item.file, change), |_, l| l, today, report)
            }
            "answer_question" => {
                let item = refs::resolve(&res, string(a, "ref")?).map_err(|e| e.to_string())?;
                let src = source(&g, &item.file);
                let change = edit::add_answer(src, item.line, string(a, "text")?)
                    .map_err(|e| edit_error(&item, e))?;
                let at = line_at(src, change.edits[0].span.start);
                let report = json!({
                    "file": g.display(&item.file),
                    "line": at,
                    "text": line_of(&change.source, at),
                });
                let file = item.file.clone();
                let moved = move |f: &str, l: usize| if f == file && l >= at { l + 1 } else { l };
                self.commit(&g, &res, one(&item.file, change), moved, today, report)
            }
            "add_items" => self.add_items(&g, &res, a, today),
            _ => unreachable!("tools/call checks the name"),
        }
    }

    fn add_items(
        &mut self,
        g: &Loaded,
        res: &Resolution,
        a: &Value,
        today: Date,
    ) -> Result<Value, String> {
        let items = strings(a, "items")?;
        if items.is_empty() {
            return Err("items names nothing to add".into());
        }
        if self.added + items.len() > self.max_new {
            return Err(format!(
                "adding {} item(s) would pass this session's limit of {} new items ({} added \
                 so far); a person can raise it with `sprig mcp --max-new-items N`",
                items.len(),
                self.max_new,
                self.added
            ));
        }
        let place = match (a["under"].as_str(), a["after"].as_str()) {
            (Some(_), Some(_)) => return Err("give under or after, not both".into()),
            (Some(r), None) => Some((r, Place::Child)),
            (None, Some(r)) => Some((r, Place::Sibling)),
            (None, None) => None,
        };
        let lines: Vec<String> = items.iter().map(|t| item_line(t, today)).collect();
        let (file, src, mut line) = match place {
            Some((r, place)) => {
                let item = refs::resolve(res, r).map_err(|e| e.to_string())?;
                let src = source(g, &item.file).to_string();
                let first = edit::add_item(&src, item.line, place, &lines[0])
                    .map_err(|e| edit_error(&item, e))?;
                let line = added_line(&src, &first);
                (item.file, first.source, line)
            }
            None => {
                let file = "PLAN.sprig".to_string();
                let src = g
                    .workspace
                    .files()
                    .find(|(k, _)| *k == file)
                    .map(|(_, s)| s.to_string())
                    .ok_or("this folder has no PLAN.sprig; give under or after")?;
                let out = appended(&src, &lines[0]);
                let line = out.lines().count();
                (file, out, line)
            }
        };
        // The rest follow the first, each after the one before.
        let first_line = line;
        let mut src = src;
        for text in &lines[1..] {
            let c = edit::add_item(&src, line, Place::Sibling, text).map_err(|e| e.to_string())?;
            line = added_line(&src, &c);
            src = c.source;
        }
        let added: Vec<Value> = (first_line..=line)
            .map(|l| json!({ "file": g.display(&file), "line": l, "text": line_of(&src, l) }))
            .collect();
        let n = lines.len();
        let (moved_file, moved_from) = (file.clone(), first_line);
        let moved = move |f: &str, l: usize| {
            if f == moved_file && l >= moved_from {
                l + n
            } else {
                l
            }
        };
        let out = self.commit(
            g,
            res,
            BTreeMap::from([(file, src)]),
            moved,
            today,
            json!({ "added": added }),
        )?;
        self.added += n;
        Ok(out)
    }

    /// The folder, read afresh. Paths in results are relative to it, as refs
    /// are, wherever the server was started; `self.root` is where they are.
    fn load(&self) -> Result<Loaded, String> {
        let mut groups = load::load(std::slice::from_ref(&self.root)).map_err(|e| e.to_string())?;
        let mut g = groups.pop().expect("one path gives one workspace");
        g.root = PathBuf::from(".");
        Ok(g)
    }

    /// Writes new texts for some files, unless that would add an error
    /// `check` reports, and returns `report` with the diff and what stopped
    /// waiting. `moved` maps a line before the change to the same line after.
    fn commit(
        &self,
        g: &Loaded,
        before: &Resolution,
        texts: BTreeMap<String, String>,
        moved: impl Fn(&str, usize) -> usize,
        today: Date,
        mut report: Value,
    ) -> Result<Value, String> {
        let mut ws: Workspace = g.workspace.clone();
        for (file, text) in &texts {
            ws.insert(file.clone(), text.clone());
        }
        let after = ws.resolve(today);
        let new = new_errors(&before.diagnostics, &after.diagnostics);
        if !new.is_empty() {
            let lines: Vec<String> = new
                .iter()
                .map(|d| {
                    format!(
                        "{}:{}:{}: error[{}]: {}",
                        g.display(&d.file),
                        d.line,
                        d.col,
                        d.code,
                        d.message
                    )
                })
                .collect();
            return Err(format!(
                "refused, and nothing was written: the change would add {}\n{}",
                if new.len() == 1 { "an error" } else { "errors" },
                lines.join("\n")
            ));
        }
        let mut diff = String::new();
        for (file, text) in &texts {
            let old = source(g, file);
            if old == text {
                continue;
            }
            let shown = g.display(file);
            diff.push_str(
                &TextDiff::from_lines(old, text.as_str())
                    .unified_diff()
                    .context_radius(1)
                    .header(&format!("a/{shown}"), &format!("b/{shown}"))
                    .to_string(),
            );
        }
        for (file, text) in &texts {
            let path = self.root.join(file);
            save(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        }
        let freed: Vec<Value> = freed(before, &after, moved)
            .iter()
            .map(|f| {
                json!({
                    "file": g.display(&f.node.file),
                    "line": f.node.line,
                    "text": f.node.label,
                    "now": f.rest(),
                })
            })
            .collect();
        let m = report.as_object_mut().expect("a report is an object");
        m.insert("diff".into(), json!(diff));
        m.insert("no_longer_waiting".into(), json!(freed));
        Ok(report)
    }
}

/// The errors in `after` that `before` does not have, matched by code and
/// message, since a line number moves when a line is added above it.
fn new_errors<'d>(before: &[Diagnostic], after: &'d [Diagnostic]) -> Vec<&'d Diagnostic> {
    let mut had: HashMap<(&str, &str, &str), usize> = HashMap::new();
    for d in before.iter().filter(|d| d.severity() == Severity::Error) {
        *had.entry((d.code.as_str(), &d.file, &d.message))
            .or_default() += 1;
    }
    after
        .iter()
        .filter(|d| d.severity() == Severity::Error)
        .filter(
            |d| match had.get_mut(&(d.code.as_str(), d.file.as_str(), d.message.as_str())) {
                Some(n) if *n > 0 => {
                    *n -= 1;
                    false
                }
                _ => true,
            },
        )
        .collect()
}

fn one(file: &str, change: Change) -> BTreeMap<String, String> {
    BTreeMap::from([(file.to_string(), change.source)])
}

fn source<'g>(g: &'g Loaded, file: &str) -> &'g str {
    g.workspace
        .files()
        .find(|(k, _)| *k == file)
        .map(|(_, s)| s)
        .expect("a resolved file is loaded")
}

fn line_at(src: &str, offset: usize) -> usize {
    src[..offset].matches('\n').count() + 1
}

/// The line an added item lands on: after the line ending inserted before
/// it, or on the line the insertion point starts.
fn added_line(src: &str, change: &Change) -> usize {
    let at = line_at(src, change.edits[0].span.start);
    if change.edits[0].text.starts_with(['\n', '\r']) {
        at + 1
    } else {
        at
    }
}

fn line_of(src: &str, line: usize) -> String {
    src.lines().nth(line - 1).unwrap_or("").trim().to_string()
}

fn edit_error(item: &refs::Item, e: edit::EditError) -> String {
    format!("{}:{}: {e}", item.file, item.line)
}

fn string<'a>(a: &'a Value, key: &str) -> Result<&'a str, String> {
    a[key]
        .as_str()
        .ok_or_else(|| format!("{key} is required, as a string"))
}

fn strings(a: &Value, key: &str) -> Result<Vec<String>, String> {
    match &a[key] {
        Value::Array(v) => v
            .iter()
            .map(|s| s.as_str().map(String::from))
            .collect::<Option<_>>()
            .ok_or_else(|| format!("{key} is a list of strings")),
        // One string is the list of one a model often sends.
        Value::String(s) => Ok(vec![s.clone()]),
        _ => Err(format!("{key} is required, as a list of strings")),
    }
}

fn pretty(v: &Value) -> String {
    serde_json::to_string_pretty(v).expect("a Value serialises")
}

fn ok(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error(id: Value, code: i32, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

const REF: &str = "An item: ^anchor, file^anchor, file.sprig:LINE, or words from its text that \
match one item. Refs come back in every result.";

/// The tools, with their argument schemas (spec/tools.md T6).
pub fn tools() -> Vec<Value> {
    let read = json!({ "readOnlyHint": true, "openWorldHint": false });
    let write = |idempotent: bool| {
        json!({
            "readOnlyHint": false,
            "destructiveHint": false,
            "idempotentHint": idempotent,
            "openWorldHint": false,
        })
    };
    let none = json!({ "type": "object", "additionalProperties": false });
    vec![
        json!({
            "name": "list_plans",
            "title": "List plans",
            "description": "Every plan in the folder with its progress: done and total, how \
                many items are ready and waiting, how many are late, and its errors. The same as \
                `sprig status --json`.",
            "inputSchema": none,
            "annotations": read,
        }),
        json!({
            "name": "show_tree",
            "title": "Show a plan",
            "description": "One plan as a resolved tree: every item with its status, progress, \
                people, dates and what it waits on, grafts mounted. The tree format of \
                spec/tree.schema.json.",
            "inputSchema": {
                "type": "object",
                "properties": { "file": { "type": "string", "description": "The plan, as list_plans names it, with or without .sprig" } },
                "required": ["file"],
                "additionalProperties": false,
            },
            "annotations": read,
        }),
        json!({
            "name": "next_items",
            "title": "What to do next",
            "description": "Open work with nothing in its way, in progress first, then by \
                priority and due date. Call this to decide what to work on. The same as \
                `sprig next --json`.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "who": { "type": "string", "description": "Only work this person owns, with or without the @" },
                    "limit": { "type": "integer", "minimum": 1, "description": "At most this many" },
                },
                "additionalProperties": false,
            },
            "annotations": read,
        }),
        json!({
            "name": "find_items",
            "title": "Find items",
            "description": "Items matching every term of a query: words from the text, @who, \
                #tag, is:open|todo|doing|done|dropped|later|ask|blocked, and due:DATE, \
                due:<DATE, due:<=DATE, due:>DATE or due:>=DATE, where DATE may be a shortcut \
                such as fri or +2w. The same as `sprig find --json`.",
            "inputSchema": {
                "type": "object",
                "properties": { "query": { "type": "string", "description": "For example: is:blocked @dana" } },
                "required": ["query"],
                "additionalProperties": false,
            },
            "annotations": read,
        }),
        json!({
            "name": "check_plans",
            "title": "Check plans",
            "description": "Every problem in the folder's plans: broken grafts, loops, \
                unknown references and the rest, with file, line and a stable code. The same as \
                `sprig check --json`. Run it before saying the plan is done.",
            "inputSchema": none,
            "annotations": read,
        }),
        json!({
            "name": "tick_items",
            "title": "Tick items",
            "description": "Mark items done, or reopen done ones; a recurring item's due date \
                moves to its next date instead. Returns the diff and what stopped waiting.",
            "inputSchema": {
                "type": "object",
                "properties": { "refs": { "type": "array", "items": { "type": "string" }, "minItems": 1, "description": REF } },
                "required": ["refs"],
                "additionalProperties": false,
            },
            "annotations": write(false),
        }),
        json!({
            "name": "mark_item",
            "title": "Set an item's mark",
            "description": "Set an item's status mark: - to do, ~ doing, x done, / dropped, \
                > later, ? question (or the words todo, doing, done, dropped, later, ask). A \
                group or a graft takes its status from its children and cannot be marked.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ref": { "type": "string", "description": REF },
                    "mark": { "type": "string", "enum": ["-", "~", "x", "/", ">", "?", "todo", "doing", "done", "dropped", "later", "ask"] },
                },
                "required": ["ref", "mark"],
                "additionalProperties": false,
            },
            "annotations": write(true),
        }),
        json!({
            "name": "add_items",
            "title": "Add items",
            "description": "Add items, in order, under an item (as its last children) or \
                after one (as siblings), indented to match; with neither, at the end of \
                PLAN.sprig. A line without a mark becomes a to-do, and typed dates such as \
                due:fri are written out. Each call and each session may add a limited number of \
                items.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "items": { "type": "array", "items": { "type": "string" }, "minItems": 1, "description": "One line each, such as \"Write the migration  due:fri  @me\"" },
                    "under": { "type": "string", "description": REF },
                    "after": { "type": "string", "description": REF },
                },
                "required": ["items"],
                "additionalProperties": false,
            },
            "annotations": write(false),
        }),
        json!({
            "name": "answer_question",
            "title": "Answer a question",
            "description": "Settle a ? item with an answer line under it (= ...). Only answer \
                a question a person asked you to settle; a question that names a person is \
                theirs.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ref": { "type": "string", "description": REF },
                    "text": { "type": "string", "description": "The answer" },
                },
                "required": ["ref", "text"],
                "additionalProperties": false,
            },
            "annotations": write(false),
        }),
        json!({
            "name": "set_field",
            "title": "Set a field",
            "description": "Set a key:value field on an item, such as due, start, est, every \
                or one of your own; an empty value removes it. A value with spaces is quoted.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "ref": { "type": "string", "description": REF },
                    "key": { "type": "string" },
                    "value": { "type": "string", "description": "Empty to remove the field" },
                },
                "required": ["ref", "key", "value"],
                "additionalProperties": false,
            },
            "annotations": write(true),
        }),
    ]
}

/// The snippet that registers this server with an MCP client.
pub fn config(root: &Path) -> Value {
    let exe = std::env::current_exe().map_or_else(|_| "sprig".into(), |p| p.display().to_string());
    let root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    json!({
        "mcpServers": {
            "sprig": { "command": exe, "args": ["mcp", "--in", root.display().to_string()] }
        }
    })
}
