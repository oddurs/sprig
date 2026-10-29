//! The language server, driven the way an editor drives it: JSON-RPC
//! messages over a connection, against a copy of `examples/bakery/` on disk.

use std::fs;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use lsp_server::{Connection, Message, Notification, Request, RequestId};
use lsp_types::notification::Notification as _;
use lsp_types::request::Request as _;
use lsp_types::{self as lsp, Position};
use serde_json::{Value, json};
use sprig_core::Date;

const TODAY: &str = "2026-10-01";

struct Client {
    conn: Connection,
    server: Option<JoinHandle<()>>,
    dir: tempfile::TempDir,
    next: i32,
    /// Notifications and requests from the server, in arrival order.
    inbox: Vec<Message>,
    /// What the client has open, which wins over the disk.
    open: std::collections::BTreeMap<String, String>,
}

fn bakery() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/bakery")
}

impl Client {
    /// A server over a copy of the bakery, initialized with these client
    /// capabilities.
    fn start(capabilities: Value) -> Client {
        let dir = tempfile::tempdir().unwrap();
        for e in fs::read_dir(bakery()).unwrap() {
            let e = e.unwrap();
            fs::copy(e.path(), dir.path().join(e.file_name())).unwrap();
        }
        Client::start_in(dir, capabilities)
    }

    fn start_in(dir: tempfile::TempDir, capabilities: Value) -> Client {
        let (server, conn) = Connection::memory();
        let handle = std::thread::spawn(move || {
            let today = Date::parse_iso(TODAY).unwrap();
            sprig_lsp::run(&server, &move || today).unwrap();
        });
        let mut c = Client {
            conn,
            server: Some(handle),
            dir,
            next: 0,
            inbox: Vec::new(),
            open: Default::default(),
        };
        let root = c.uri_of(c.dir.path());
        let init = c.request(
            "initialize",
            json!({"processId": null, "rootUri": root, "capabilities": capabilities}),
        );
        assert_eq!(init["serverInfo"]["name"], "sprig");
        c.notify("initialized", json!({}));
        c
    }

    fn uri_of(&self, path: &Path) -> String {
        url::Url::from_file_path(path).unwrap().to_string()
    }

    fn uri(&self, name: &str) -> String {
        self.uri_of(&self.dir.path().join(name))
    }

    fn text(&self, name: &str) -> String {
        self.open
            .get(name)
            .cloned()
            .unwrap_or_else(|| self.disk(name))
    }

    fn disk(&self, name: &str) -> String {
        fs::read_to_string(self.dir.path().join(name)).unwrap()
    }

    fn notify(&self, method: &str, params: Value) {
        self.conn
            .sender
            .send(Message::Notification(Notification::new(
                method.into(),
                params,
            )))
            .unwrap();
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next += 1;
        let id = RequestId::from(self.next);
        self.conn
            .sender
            .send(Message::Request(Request::new(
                id.clone(),
                method.into(),
                params,
            )))
            .unwrap();
        loop {
            match self
                .conn
                .receiver
                .recv_timeout(Duration::from_secs(10))
                .expect("the server answers")
            {
                Message::Response(r) if r.id == id => {
                    assert!(r.error.is_none(), "{method} failed: {:?}", r.error);
                    return r.result.unwrap_or(Value::Null);
                }
                other => self.inbox.push(other),
            }
        }
    }

    fn open(&mut self, name: &str, text: &str) {
        self.open.insert(name.to_string(), text.to_string());
        self.notify(
            "textDocument/didOpen",
            json!({"textDocument": {"uri": self.uri(name), "languageId": "sprig", "version": 1, "text": text}}),
        );
    }

    fn change(&mut self, name: &str, text: &str) {
        self.open.insert(name.to_string(), text.to_string());
        self.notify(
            "textDocument/didChange",
            json!({"textDocument": {"uri": self.uri(name), "version": 2}, "contentChanges": [{"text": text}]}),
        );
    }

    /// Everything the server sends until it has been quiet for a moment:
    /// diagnostics are published once its queue is empty.
    fn settle(&mut self) -> Vec<Message> {
        let mut out = std::mem::take(&mut self.inbox);
        while let Ok(m) = self.conn.receiver.recv_timeout(Duration::from_millis(300)) {
            out.push(m);
        }
        out
    }

    /// The diagnostics last published for each file, by file name.
    fn diagnostics(&mut self) -> Vec<(String, Vec<lsp::Diagnostic>)> {
        let mut out: Vec<(String, Vec<lsp::Diagnostic>)> = Vec::new();
        for m in self.settle() {
            if let Message::Notification(n) = m
                && n.method == lsp::notification::PublishDiagnostics::METHOD
            {
                let p: lsp::PublishDiagnosticsParams = serde_json::from_value(n.params).unwrap();
                let name = p.uri.as_str().rsplit('/').next().unwrap().to_string();
                out.retain(|(f, _)| *f != name);
                out.push((name, p.diagnostics));
            }
        }
        out
    }

    fn at(&self, name: &str, needle: &str, nudge: u32) -> Value {
        let text = self.text(name);
        let at = text
            .find(needle)
            .unwrap_or_else(|| panic!("{needle} in {name}"));
        let line = text[..at].matches('\n').count() as u32;
        let col = text[..at]
            .rsplit('\n')
            .next()
            .unwrap()
            .encode_utf16()
            .count() as u32;
        json!({"textDocument": {"uri": self.uri(name)}, "position": {"line": line, "character": col + nudge}})
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        let _ = self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        if let Some(h) = self.server.take() {
            h.join().unwrap();
        }
    }
}

fn codes(diags: &[lsp::Diagnostic]) -> Vec<(u32, String)> {
    diags
        .iter()
        .map(|d| {
            let code = match &d.code {
                Some(lsp::NumberOrString::String(s)) => s.clone(),
                _ => String::new(),
            };
            (d.range.start.line + 1, code)
        })
        .collect()
}

fn of<'a>(
    published: &'a [(String, Vec<lsp::Diagnostic>)],
    file: &str,
) -> Option<&'a [lsp::Diagnostic]> {
    published
        .iter()
        .find(|(f, _)| f == file)
        .map(|(_, d)| d.as_slice())
}

// ----------------------------------------------------------------------------
// Diagnostics (cairn 0049)

#[test]
fn diagnostics_appear_on_open_and_follow_each_change() {
    let mut c = Client::start(json!({}));
    let clean = c.diagnostics();
    assert!(clean.is_empty(), "the bakery is clean: {clean:?}");

    let broken = c.disk("launch.sprig") + "- Posters  after:^nowhere  due:soonish\n";
    c.open("launch.sprig", &broken);
    let found = c.diagnostics();
    let line = broken.lines().count() as u32;
    assert_eq!(
        codes(of(&found, "launch.sprig").unwrap()),
        [(line, "S0201".to_string()), (line, "S0001".to_string())]
    );

    let clean = c.disk("launch.sprig");
    c.change("launch.sprig", &clean);
    let fixed = c.diagnostics();
    assert_eq!(
        of(&fixed, "launch.sprig"),
        Some(&[][..]),
        "cleared when fixed"
    );
}

#[test]
fn deleting_an_anchor_reports_the_files_that_name_it_without_editing_them() {
    let mut c = Client::start(json!({}));
    c.diagnostics();
    let lease = c.text("lease.sprig");
    c.open("lease.sprig", &lease);
    c.change("lease.sprig", &lease.replace("  ^signed", ""));
    let found = c.diagnostics();
    assert_eq!(
        codes(of(&found, "bakery.sprig").expect("bakery.sprig is republished")),
        [(15, "S0203".to_string())]
    );
    assert_eq!(
        codes(of(&found, "kitchen.sprig").expect("kitchen.sprig is republished")),
        [(18, "S0203".to_string())]
    );
    assert!(
        of(&found, "lease.sprig").is_none(),
        "lease.sprig itself is fine"
    );

    c.change("lease.sprig", &lease);
    let healed = c.diagnostics();
    assert_eq!(of(&healed, "bakery.sprig"), Some(&[][..]));
    assert_eq!(of(&healed, "kitchen.sprig"), Some(&[][..]));
}

#[test]
fn a_file_changed_on_disk_is_read_again_when_the_client_reports_it() {
    let mut c = Client::start(
        json!({"workspace": {"didChangeWatchedFiles": {"dynamicRegistration": true}}}),
    );
    let registered = c.settle().into_iter().any(|m| {
        matches!(&m, Message::Request(r) if r.method == lsp::request::RegisterCapability::METHOD)
    });
    assert!(registered, "the server asks to watch *.sprig");

    let path = c.dir.path().join("lease.sprig");
    fs::write(&path, c.text("lease.sprig").replace("  ^signed", "")).unwrap();
    c.notify(
        "workspace/didChangeWatchedFiles",
        json!({"changes": [{"uri": c.uri("lease.sprig"), "type": 2}]}),
    );
    let found = c.diagnostics();
    assert_eq!(
        codes(of(&found, "bakery.sprig").unwrap()),
        [(15, "S0203".to_string())]
    );
}

#[test]
fn rechecking_a_large_workspace_after_a_keystroke_is_quick() {
    // 5,000 lines in 50 files, shaped like the bakery: one plan grafting the
    // other 49, every item naming an anchor in the next file, so a keystroke
    // in the plan touches every tree.
    let dir = tempfile::tempdir().unwrap();
    for i in 0..50 {
        let mut s = format!("Part {i}\n\n");
        if i == 0 {
            for j in 1..50 {
                s += &format!("+ [[part{j}]]\n");
            }
        }
        while s.lines().count() < 100 {
            let n = s.lines().count();
            s += &format!(
                "- Task {n}  ^t{n}  est:2h  @p{}  after:[[part{}^t{n}]]\n  - Step  due:2026-11-0{}\n",
                n % 7,
                (i + 1) % 50,
                n % 9 + 1
            );
        }
        fs::write(dir.path().join(format!("part{i}.sprig")), s).unwrap();
    }
    let mut c = Client::start_in(dir, json!({}));
    let text = c.text("part0.sprig");
    c.open("part0.sprig", &text);
    c.diagnostics();

    let mut times = Vec::new();
    for k in 0..9 {
        let edited = format!("{text}- typed {k}  after:^missing{k}\n");
        let start = Instant::now();
        c.change("part0.sprig", &edited);
        loop {
            let m = c
                .conn
                .receiver
                .recv_timeout(Duration::from_secs(10))
                .unwrap();
            if let Message::Notification(n) = m
                && n.method == lsp::notification::PublishDiagnostics::METHOD
                && n.params["uri"].as_str().unwrap().ends_with("part0.sprig")
            {
                break;
            }
        }
        times.push(start.elapsed());
        c.settle();
    }
    times.sort();
    let (median, worst) = (times[times.len() / 2], times[times.len() - 1]);
    eprintln!("keystroke recheck of 5,000 lines: median {median:?}, worst {worst:?}");
    // The criterion is 50 ms on a CI runner in a release build; a debug
    // build is held to ten times that, so a regression still shows.
    let limit = if cfg!(debug_assertions) { 500 } else { 50 };
    assert!(
        median < Duration::from_millis(limit),
        "a keystroke took {median:?} (median of {}) to recheck 5,000 lines",
        times.len()
    );
}

// ----------------------------------------------------------------------------
// Completion (cairn 0050)

fn labels(v: &Value) -> Vec<String> {
    let mut out: Vec<String> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["label"].as_str().unwrap().to_string())
        .collect();
    out.sort();
    out
}

/// Opens `name` with `line` added at the end and asks for completion at the
/// end of it.
fn complete(c: &mut Client, name: &str, line: &str) -> Value {
    let text = c.disk(name) + line;
    c.open(name, &text);
    let last = text.lines().count() as u32 - 1;
    let col = line.trim_end_matches('\n').encode_utf16().count() as u32;
    c.request(
        "textDocument/completion",
        json!({"textDocument": {"uri": c.uri(name)}, "position": {"line": last, "character": col}}),
    )
}

#[test]
fn completing_a_person_offers_everyone_in_the_workspace() {
    let mut c = Client::start(json!({}));
    let got = complete(&mut c, "launch.sprig", "- Flyers  @");
    assert_eq!(labels(&got), ["@dana", "@kim", "@teo"]);
    let edit = &got[0]["textEdit"];
    assert_eq!(
        edit["range"]["start"]["character"], 10,
        "replaces the @ typed"
    );
}

#[test]
fn completing_an_anchor_offers_this_files_anchors() {
    let mut c = Client::start(json!({}));
    let got = complete(&mut c, "launch.sprig", "- Flyers  after:^");
    assert_eq!(labels(&got), ["counter", "logo"]);
    assert!(got[0]["detail"].as_str().is_some(), "with the item's text");
    let got = complete(&mut c, "launch.sprig", "- Flyers  ^c");
    assert_eq!(labels(&got), ["counter", "logo"], "not the one being typed");
}

#[test]
fn completing_a_link_offers_the_other_files() {
    let mut c = Client::start(json!({}));
    let got = complete(&mut c, "bakery.sprig", "+ [[");
    assert_eq!(
        labels(&got),
        ["dana", "groceries", "kitchen", "launch", "lease"]
    );
    let kitchen = got
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["label"] == "kitchen")
        .unwrap();
    assert_eq!(kitchen["textEdit"]["newText"], "kitchen]]");
}

#[test]
fn completing_a_files_anchor_offers_exactly_that_files_anchors() {
    let mut c = Client::start(json!({}));
    let got = complete(&mut c, "bakery.sprig", "- Deposit  after:[[lease^");
    let mut expected = Vec::new();
    let lease = sprig_core::parse("lease.sprig", &c.text("lease.sprig")).document;
    sprig_core::syntax::walk(&lease.root, &mut |n: &sprig_core::Node| {
        if let Some(a) = n.tokens.anchor() {
            expected.push(a.name.clone());
        }
    });
    expected.sort();
    assert_eq!(labels(&got), expected);
    assert_eq!(expected, ["permit", "signed", "terms"]);
}

#[test]
fn completing_a_field_key_offers_the_keys_that_mean_something() {
    let mut c = Client::start(json!({}));
    let got = complete(&mut c, "launch.sprig", "- Flyers  d");
    assert_eq!(labels(&got), ["due:"]);
    let got = complete(&mut c, "launch.sprig", "- Flyers  e");
    assert_eq!(labels(&got), ["est:", "every:"]);
    let got = complete(&mut c, "launch.sprig", "- Flyers  z");
    assert_eq!(labels(&got), Vec::<String>::new());
}

#[test]
fn nothing_is_offered_in_notes_or_comments() {
    let mut c = Client::start(json!({}));
    let got = complete(&mut c, "launch.sprig", "  a note about @");
    assert_eq!(labels(&got), Vec::<String>::new());
    let got = complete(&mut c, "launch.sprig", "// a comment about @");
    assert_eq!(labels(&got), Vec::<String>::new());
}

// ----------------------------------------------------------------------------
// Definition and references (cairn 0051)

#[test]
fn definition_on_a_link_opens_the_item_it_names() {
    let mut c = Client::start(json!({}));
    let params = c.at("bakery.sprig", "after:[[lease^signed]]", 12);
    let got = c.request("textDocument/definition", params);
    let got = got.as_array().unwrap();
    assert_eq!(got.len(), 1);
    assert!(got[0]["uri"].as_str().unwrap().ends_with("/lease.sprig"));
    assert_eq!(
        got[0]["range"]["start"]["line"], 12,
        "lease.sprig:13, Sign the lease"
    );

    let params = c.at("bakery.sprig", "+ [[kitchen]]", 4);
    let got = c.request("textDocument/definition", params);
    assert!(got[0]["uri"].as_str().unwrap().ends_with("/kitchen.sprig"));
    assert_eq!(got[0]["range"]["start"]["line"], 0, "the title");

    let params = c.at("lease.sprig", "after:^terms", 8);
    let got = c.request("textDocument/definition", params);
    assert_eq!(
        got[0]["range"]["start"]["line"], 5,
        "lease.sprig:6, in the same file"
    );
}

#[test]
fn references_on_an_anchor_list_every_line_that_names_it() {
    let mut c = Client::start(json!({}));
    let mut params = c.at("lease.sprig", "^signed", 2);
    params["context"] = json!({"includeDeclaration": false});
    let got = c.request("textDocument/references", params.clone());
    let mut found: Vec<(String, u64)> = got
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["uri"]
                    .as_str()
                    .unwrap()
                    .rsplit('/')
                    .next()
                    .unwrap()
                    .to_string(),
                l["range"]["start"]["line"].as_u64().unwrap() + 1,
            )
        })
        .collect();
    found.sort();
    // bakery.sprig:14 mentions it in a note, which is not a reference (§2.6).
    assert_eq!(
        found,
        [
            ("bakery.sprig".to_string(), 15),
            ("kitchen.sprig".to_string(), 18)
        ]
    );

    params["context"] = json!({"includeDeclaration": true});
    let got = c.request("textDocument/references", params);
    assert_eq!(got.as_array().unwrap().len(), 3, "and the anchor itself");

    // The same answer from a reference as from the anchor.
    let mut params = c.at("kitchen.sprig", "[[lease^signed]]", 3);
    params["context"] = json!({"includeDeclaration": false});
    let got = c.request("textDocument/references", params);
    assert_eq!(got.as_array().unwrap().len(), 2);
}

// ----------------------------------------------------------------------------
// Hover (cairn 0052)

fn hover_text(c: &mut Client, name: &str, needle: &str) -> String {
    let params = c.at(name, needle, 1);
    let got = c.request("textDocument/hover", params);
    got["contents"]["value"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn hovering_a_graft_shows_the_grafted_files_totals() {
    let mut c = Client::start(json!({}));
    let mut ws = sprig_core::Workspace::new();
    for f in ["bakery", "kitchen", "lease", "launch", "dana", "groceries"] {
        ws.insert(format!("{f}.sprig"), c.text(&format!("{f}.sprig")));
    }
    let res = ws.resolve(Date::parse_iso(TODAY).unwrap());
    let p = res.tree("kitchen.sprig").unwrap().root.progress;
    let shown = hover_text(&mut c, "bakery.sprig", "+ [[kitchen]]");
    assert!(
        shown.contains(&format!("{} of {} done", p.done, p.total)),
        "{shown}"
    );
    assert!(shown.contains("@teo"), "{shown}");
    assert!(shown.contains("Mounts `kitchen.sprig`"), "{shown}");
}

#[test]
fn hovering_a_blocked_item_names_everything_it_waits_on() {
    let mut c = Client::start(json!({}));
    let shown = hover_text(&mut c, "lease.sprig", "- Sign the lease");
    assert!(shown.contains("Waiting on:"), "{shown}");
    assert!(
        shown.contains("*Negotiate terms* (lease.sprig:6)"),
        "{shown}"
    );
    assert!(
        shown.contains("*Change-of-use permit* (lease.sprig:11)"),
        "{shown}"
    );
    let shown = hover_text(&mut c, "kitchen.sprig", "- Build");
    assert!(
        shown.contains("*Sign the lease* (lease.sprig:13)"),
        "{shown}"
    );
}

// ----------------------------------------------------------------------------
// Code actions and formatting (cairn 0053)

fn actions(c: &mut Client, name: &str, needle: &str) -> Vec<Value> {
    let p = c.at(name, needle, 0);
    let pos = p["position"].clone();
    let got = c.request(
        "textDocument/codeAction",
        json!({"textDocument": p["textDocument"], "range": {"start": pos, "end": pos}, "context": {"diagnostics": []}}),
    );
    got.as_array().cloned().unwrap_or_default()
}

fn edits(action: &Value) -> Vec<(lsp::Range, String)> {
    let changes = action["edit"]["changes"].as_object().unwrap();
    assert_eq!(changes.len(), 1, "one file");
    changes
        .values()
        .next()
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                serde_json::from_value(e["range"].clone()).unwrap(),
                e["newText"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn titled<'a>(actions: &'a [Value], title: &str) -> &'a Value {
    actions
        .iter()
        .find(|a| a["title"].as_str().unwrap().starts_with(title))
        .unwrap_or_else(|| panic!("no {title:?} in {actions:#?}"))
}

fn one_char(line: u32, col: u32) -> lsp::Range {
    lsp::Range::new(Position::new(line, col), Position::new(line, col + 1))
}

#[test]
fn ticking_touches_only_the_mark() {
    let mut c = Client::start(json!({}));
    let got = actions(&mut c, "bakery.sprig", "- Supplier credit line");
    assert_eq!(
        edits(titled(&got, "Mark done")),
        [(one_char(14, 2), "x".into())]
    );
    assert_eq!(
        edits(titled(&got, "Start: mark doing")),
        [(one_char(14, 2), "~".into())]
    );
    let got = actions(&mut c, "bakery.sprig", "x Bank meeting");
    assert_eq!(
        edits(titled(&got, "Reopen")),
        [(one_char(11, 2), "-".into())]
    );
}

#[test]
fn ticking_a_recurring_item_rolls_its_due_date_instead_of_closing_it() {
    let mut c = Client::start(json!({}));
    let got = actions(&mut c, "bakery.sprig", "- Weekly numbers to the bank");
    let tick = titled(&got, "Done for now: move due to 2026-10-09");
    let line = c.text("bakery.sprig").lines().nth(15).unwrap().to_string();
    let date = line.find("2026-10-02").unwrap() as u32;
    let mut after = line.clone();
    for (r, text) in edits(tick).into_iter().rev() {
        assert_eq!(r.start.line, 15);
        assert!(
            r.start.character >= date && r.end.character <= date + 10,
            "only the date changes: {r:?}"
        );
        after.replace_range(r.start.character as usize..r.end.character as usize, &text);
    }
    assert_eq!(after, line.replace("2026-10-02", "2026-10-09"));
    assert!(
        got.iter()
            .all(|a| !a["title"].as_str().unwrap().starts_with("Finish")),
        "a recurring item is never simply marked done"
    );
}

#[test]
fn a_typed_date_is_written_out_touching_only_its_value() {
    let mut c = Client::start(json!({}));
    let text = c.disk("launch.sprig") + "- Flyers  due:fri  @kim\n";
    c.open("launch.sprig", &text);
    let got = actions(&mut c, "launch.sprig", "- Flyers");
    let line = text.lines().count() as u32 - 1;
    assert_eq!(
        edits(titled(&got, "Write due:fri as 2026-10-02")),
        [(
            lsp::Range::new(Position::new(line, 14), Position::new(line, 17)),
            "2026-10-02".into()
        )]
    );
}

#[test]
fn formatting_returns_the_formatters_edits() {
    let mut c = Client::start(json!({}));
    c.open("launch.sprig", "Launch\n\n- Flyers  due:fri   \n");
    let got = c.request(
        "textDocument/formatting",
        json!({"textDocument": {"uri": c.uri("launch.sprig")}, "options": {"tabSize": 2, "insertSpaces": true}}),
    );
    let mut text = "Launch\n\n- Flyers  due:fri   \n".to_string();
    let edits: Vec<lsp::TextEdit> = serde_json::from_value(got).unwrap();
    // Apply from the end, as an editor does.
    for e in edits.iter().rev() {
        let start = offset(&text, e.range.start);
        let end = offset(&text, e.range.end);
        text.replace_range(start..end, &e.new_text);
    }
    assert_eq!(text, "Launch\n\n- Flyers  due:2026-10-02\n");
}

fn offset(text: &str, p: Position) -> usize {
    let start: usize = text
        .split_inclusive('\n')
        .take(p.line as usize)
        .map(str::len)
        .sum();
    start + p.character as usize
}

#[test]
fn a_client_that_counts_utf8_gets_utf8_positions() {
    let (server, conn) = Connection::memory();
    let h = std::thread::spawn(move || {
        let today = Date::parse_iso(TODAY).unwrap();
        sprig_lsp::run(&server, &move || today).unwrap();
    });
    conn.sender
        .send(Message::Request(Request::new(
            RequestId::from(1),
            "initialize".into(),
            json!({"processId": null, "rootUri": null, "capabilities": {"general": {"positionEncodings": ["utf-8", "utf-16"]}}}),
        )))
        .unwrap();
    let Message::Response(r) = conn.receiver.recv().unwrap() else {
        panic!("a response first")
    };
    assert_eq!(
        r.result.unwrap()["capabilities"]["positionEncoding"],
        "utf-8"
    );
    conn.sender
        .send(Message::Notification(Notification::new(
            "initialized".into(),
            json!({}),
        )))
        .unwrap();
    conn.sender
        .send(Message::Request(Request::new(
            RequestId::from(2),
            "shutdown".into(),
            Value::Null,
        )))
        .unwrap();
    conn.sender
        .send(Message::Notification(Notification::new(
            "exit".into(),
            Value::Null,
        )))
        .unwrap();
    h.join().unwrap();
}
