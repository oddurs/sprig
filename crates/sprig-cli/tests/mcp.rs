//! `sprig mcp`, driven over stdio the way an agent's client drives it: each
//! read tool answers what the matching `--json` command prints, and each
//! write changes only what it means to, refuses to break a plan, and returns
//! the diff it applied.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{Value, json};

fn repo(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn bakery() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "mcp-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for e in fs::read_dir(repo("examples/bakery")).unwrap() {
        let e = e.unwrap();
        fs::copy(e.path(), dir.join(e.file_name())).unwrap();
    }
    dir
}

fn cli(dir: &Path, args: &[&str]) -> Value {
    let o = Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(args)
        .current_dir(dir)
        .env("SPRIG_TODAY", "2026-10-01")
        .output()
        .unwrap();
    serde_json::from_slice(&o.stdout)
        .unwrap_or_else(|e| panic!("{args:?}: {e}\n{}", String::from_utf8_lossy(&o.stdout)))
}

struct Session {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    tools: Vec<Value>,
}

impl Session {
    fn start(dir: &Path, extra: &[&str]) -> Session {
        let mut child = Command::new(env!("CARGO_BIN_EXE_sprig"))
            .arg("mcp")
            .args(extra)
            // Started from elsewhere, as a client does, and pointed at the folder.
            .arg("--in")
            .arg(dir)
            .env("SPRIG_TODAY", "2026-10-01")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut s = Session {
            child,
            stdin,
            stdout,
            next_id: 0,
            tools: Vec::new(),
        };
        let init = s.request(
            "initialize",
            json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "0" },
            }),
        );
        assert_eq!(init["result"]["protocolVersion"], "2025-11-25");
        assert!(init["result"]["capabilities"]["tools"].is_object());
        s.notify("notifications/initialized");
        s.tools = s.request("tools/list", json!({}))["result"]["tools"]
            .as_array()
            .unwrap()
            .clone();
        s
    }

    fn send(&mut self, msg: &Value) {
        writeln!(self.stdin, "{msg}").unwrap();
        self.stdin.flush().unwrap();
    }

    fn notify(&mut self, method: &str) {
        self.send(&json!({ "jsonrpc": "2.0", "method": method }));
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        let mut line = String::new();
        self.stdout.read_line(&mut line).unwrap();
        let reply: Value = serde_json::from_str(&line).unwrap_or_else(|e| panic!("{e}: {line}"));
        assert_eq!(reply["id"], id, "{reply}");
        reply
    }

    /// Calls a tool with arguments its own schema accepts, and returns the
    /// result.
    fn call(&mut self, name: &str, args: Value) -> Value {
        let tool = self
            .tools
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("no tool {name}"));
        let schema = jsonschema::validator_for(&tool["inputSchema"]).unwrap();
        assert!(
            schema.is_valid(&args),
            "{name}: {args} does not fit its schema"
        );
        let reply = self.request("tools/call", json!({ "name": name, "arguments": args }));
        reply["result"].clone()
    }

    /// The JSON a successful tool returned, checked to be the same as text
    /// and as structured content.
    fn ok(&mut self, name: &str, args: Value) -> Value {
        let r = self.call(name, args);
        assert_eq!(r["isError"], false, "{name}: {r}");
        let text: Value = serde_json::from_str(r["content"][0]["text"].as_str().unwrap()).unwrap();
        let structured = &r["structuredContent"];
        assert!(structured.is_object());
        if text.is_array() {
            assert_eq!(structured["items"], text);
        } else {
            assert_eq!(structured, &text);
        }
        text
    }

    /// The message of a tool that failed.
    fn err(&mut self, name: &str, args: Value) -> String {
        let r = self.call(name, args);
        assert_eq!(r["isError"], true, "{name}: {r}");
        r["content"][0]["text"].as_str().unwrap().to_string()
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn every_tool_has_a_valid_schema_and_honest_annotations() {
    let dir = bakery();
    let s = Session::start(&dir, &[]);
    let names: Vec<&str> = s
        .tools
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "list_plans",
            "show_tree",
            "next_items",
            "find_items",
            "check_plans",
            "tick_items",
            "mark_item",
            "add_items",
            "answer_question",
            "set_field",
        ]
    );
    for t in &s.tools {
        let name = t["name"].as_str().unwrap();
        assert!(
            jsonschema::validator_for(&t["inputSchema"]).is_ok(),
            "{name}"
        );
        assert_eq!(t["inputSchema"]["type"], "object", "{name}");
        assert!(
            t["description"].as_str().is_some_and(|d| d.len() > 40),
            "{name}"
        );
        let read = [
            "list_plans",
            "show_tree",
            "next_items",
            "find_items",
            "check_plans",
        ]
        .contains(&name);
        assert_eq!(t["annotations"]["readOnlyHint"], read, "{name}");
        assert_eq!(t["annotations"]["openWorldHint"], false, "{name}");
    }
}

#[test]
fn an_older_client_is_answered_in_its_own_protocol_version() {
    let dir = bakery();
    let mut s = Session::start(&dir, &[]);
    let r = s.request(
        "initialize",
        json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "old", "version": "0" } }),
    );
    assert_eq!(r["result"]["protocolVersion"], "2025-06-18");
    let r = s.request("ping", json!({}));
    assert_eq!(r["result"], json!({}));
}

#[test]
fn read_tools_answer_what_the_json_commands_print() {
    let dir = bakery();
    let mut s = Session::start(&dir, &[]);
    assert_eq!(
        s.ok("list_plans", json!({})),
        cli(&dir, &["status", "--json"])
    );
    assert_eq!(
        s.ok("next_items", json!({})),
        cli(&dir, &["next", "--json"])
    );
    assert_eq!(
        s.ok("next_items", json!({ "who": "@teo", "limit": 2 })),
        cli(&dir, &["next", "--json", "--who", "teo", "--limit", "2"])
    );
    assert_eq!(
        s.ok("find_items", json!({ "query": "is:blocked @kim" })),
        cli(&dir, &["find", "--json", "is:blocked", "@kim"])
    );
    assert_eq!(
        s.ok("check_plans", json!({})),
        cli(&dir, &["check", "--json"])
    );
    let exported = cli(&dir, &["export", "--format", "json"]);
    let bakery = exported
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["file"] == "bakery.sprig")
        .unwrap();
    assert_eq!(&s.ok("show_tree", json!({ "file": "bakery" })), bakery);
    // A grafted plan has a tree of its own too.
    let lease = s.ok("show_tree", json!({ "file": "lease.sprig" }));
    assert_eq!(lease["file"], "lease.sprig");
}

#[test]
fn a_tool_that_fails_says_why_in_its_result() {
    let dir = bakery();
    let mut s = Session::start(&dir, &[]);
    assert!(
        s.err("show_tree", json!({ "file": "nope" }))
            .contains("list_plans")
    );
    let e = s.err("find_items", json!({ "query": "colour:red" }));
    assert!(e.contains("is not a filter"), "{e}");
    let e = s.err("tick_items", json!({ "refs": ["^nowhere"] }));
    assert!(e.contains("^nowhere"), "{e}");
    let e = s.err("mark_item", json!({ "ref": "launch.sprig:3", "mark": "x" }));
    assert!(e.contains("children"), "{e}");
    // An unknown tool is the client's mistake, not the model's.
    let r = s.request(
        "tools/call",
        json!({ "name": "delete_everything", "arguments": {} }),
    );
    assert_eq!(r["error"]["code"], -32602);
}

/// The lines of `after` that are not in `before`, and the reverse, as a
/// line-by-line comparison of two texts that differ in only a few places.
fn changed_lines(before: &str, after: &str) -> (Vec<String>, Vec<String>) {
    let diff = similar::TextDiff::from_lines(before, after);
    let mut removed = Vec::new();
    let mut added = Vec::new();
    for c in diff.iter_all_changes() {
        match c.tag() {
            similar::ChangeTag::Delete => removed.push(c.value().to_string()),
            similar::ChangeTag::Insert => added.push(c.value().to_string()),
            similar::ChangeTag::Equal => {}
        }
    }
    (removed, added)
}

#[test]
fn writes_change_only_their_lines_and_return_the_diff() {
    let dir = bakery();
    let mut s = Session::start(&dir, &[]);
    let read = |f: &str| fs::read_to_string(dir.join(f)).unwrap();

    let before = read("lease.sprig");
    let r = s.ok("tick_items", json!({ "refs": ["^signed"] }));
    let after = read("lease.sprig");
    let (removed, added) = changed_lines(&before, &after);
    assert_eq!(
        removed,
        ["- Sign the lease  ^signed  after:^terms  after:^permit\n"]
    );
    assert_eq!(
        added,
        ["x Sign the lease  ^signed  after:^terms  after:^permit\n"]
    );
    let diff = r["diff"].as_str().unwrap();
    assert!(
        diff.starts_with("--- a/lease.sprig\n+++ b/lease.sprig\n@@ "),
        "{diff}"
    );
    assert!(
        diff.contains("\n-- Sign the lease") && diff.contains("\n+x Sign the lease"),
        "{diff}"
    );
    let freed = &r["no_longer_waiting"];
    assert!(
        freed
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["text"] == "Supplier credit line" && f["now"] == "unblocked"),
        "{freed}"
    );

    let before = read("kitchen.sprig");
    let r = s.ok(
        "add_items",
        json!({ "items": ["Call the plumber due:fri", "~ Pick tiles"], "under": "^design" }),
    );
    let (removed, added) = changed_lines(&before, &read("kitchen.sprig"));
    assert!(removed.is_empty(), "{removed:?}");
    assert_eq!(
        added,
        ["  - Call the plumber due:2026-10-02\n", "  ~ Pick tiles\n"]
    );
    assert_eq!(r["added"][0]["line"], 8);
    assert_eq!(r["added"][1]["line"], 9);

    let before = read("bakery.sprig");
    let r = s.ok(
        "answer_question",
        json!({ "ref": "^name", "text": "Kiln & Crumb" }),
    );
    let (removed, added) = changed_lines(&before, &read("bakery.sprig"));
    assert!(removed.is_empty());
    assert_eq!(added, ["  = Kiln & Crumb\n"]);
    assert!(r["diff"].as_str().unwrap().contains("+  = Kiln & Crumb"));

    let before = read("launch.sprig");
    s.ok(
        "mark_item",
        json!({ "ref": "launch.sprig:7", "mark": "doing" }),
    );
    s.ok(
        "set_field",
        json!({ "ref": "^logo", "key": "est", "value": "3d" }),
    );
    let (removed, added) = changed_lines(&before, &read("launch.sprig"));
    assert_eq!(removed.len(), 2);
    assert_eq!(
        added,
        [
            "  ~ Logo, second round  ^logo  due:2026-10-14  est:3d\n",
            "  ~ Menu boards  after:^logo  est:4h\n"
        ]
    );

    // After all that, the plans still check clean.
    assert_eq!(s.ok("check_plans", json!({})), json!([]));
}

#[test]
fn a_write_that_would_break_the_plan_is_refused_with_the_diagnostic() {
    let dir = bakery();
    let before = fs::read_to_string(dir.join("launch.sprig")).unwrap();
    let mut s = Session::start(&dir, &[]);
    let e = s.err(
        "set_field",
        json!({ "ref": "^logo", "key": "after", "value": "^nowhere" }),
    );
    assert!(e.starts_with("refused, and nothing was written"), "{e}");
    assert!(
        e.contains("launch.sprig:4:") && e.contains("error[S0201]"),
        "{e}"
    );
    assert_eq!(
        fs::read_to_string(dir.join("launch.sprig")).unwrap(),
        before
    );
}

#[test]
fn adding_more_than_the_cap_is_refused_and_names_the_flag() {
    let dir = bakery();
    let mut s = Session::start(&dir, &["--max-new-items", "3"]);
    let e = s.err(
        "add_items",
        json!({ "items": ["a", "b", "c", "d"], "under": "^design" }),
    );
    assert!(
        e.contains("limit of 3") && e.contains("--max-new-items"),
        "{e}"
    );
    s.ok(
        "add_items",
        json!({ "items": ["a", "b"], "under": "^design" }),
    );
    s.ok("add_items", json!({ "items": ["c"], "after": "^design" }));
    // The session has used its three.
    let e = s.err("add_items", json!({ "items": ["d"], "after": "^design" }));
    assert!(e.contains("3 added so far"), "{e}");
}

#[test]
fn the_default_cap_is_in_the_help() {
    let o = Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(["mcp", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&o.stdout);
    assert!(
        help.contains("--max-new-items <N>") && help.contains("[default: 20]"),
        "{help}"
    );
}

#[test]
fn config_prints_a_snippet_a_client_can_load() {
    let dir = bakery();
    let o = Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(["mcp", "--config", "--in"])
        .arg(&dir)
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    let server = &v["mcpServers"]["sprig"];
    assert!(server["command"].as_str().unwrap().contains("sprig"));
    assert_eq!(server["args"][0], "mcp");
    assert_eq!(server["args"][1], "--in");
}

#[test]
fn agent_write_keeps_one_block_and_a_second_run_changes_nothing() {
    let dir = bakery();
    let file = dir.join("AGENTS.md");
    fs::write(&file, "# Agents\n\nRun the tests before pushing.\n").unwrap();
    let run = || {
        let o = Command::new(env!("CARGO_BIN_EXE_sprig"))
            .args(["agent", "--write"])
            .arg(&file)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        fs::read_to_string(&file).unwrap()
    };
    let once = run();
    assert!(once.starts_with("# Agents\n\nRun the tests before pushing.\n\n<!-- sprig:start"));
    assert!(once.contains("`PLAN.sprig` at the root"));
    assert_eq!(run(), once);
    // Printed, it is the same text without the markers.
    let o = Command::new(env!("CARGO_BIN_EXE_sprig"))
        .arg("agent")
        .output()
        .unwrap();
    let printed = String::from_utf8(o.stdout).unwrap();
    assert!(once.contains(&printed));
}
