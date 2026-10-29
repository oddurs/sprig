//! The `sprig` binary, run the way a person or a CI job runs it.
//!
//! `SPRIG_BLESS=1 cargo test -p sprig-cli` rewrites the snapshots under
//! `tests/snapshots/`; read the diff before committing one.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

fn repo(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn sprig_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(args)
        .current_dir(dir)
        .env_remove("SPRIG_TODAY")
        .output()
        .expect("the sprig binary runs")
}

fn sprig(args: &[&str]) -> Output {
    sprig_in(&repo(""), args)
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// A fresh directory for one test, under the target directory so nothing
/// outside the build is touched.
fn scratch(name: &str) -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "{name}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn copy_bakery(to: &Path) {
    for e in fs::read_dir(repo("examples/bakery")).unwrap() {
        let p = e.unwrap().path();
        fs::copy(&p, to.join(p.file_name().unwrap())).unwrap();
    }
}

fn snapshot(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name);
    if std::env::var_os("SPRIG_BLESS").is_some() {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("no snapshot {}; run with SPRIG_BLESS=1", path.display()));
    assert_eq!(actual, expected.replace("\r\n", "\n"), "snapshot {name}");
}

#[test]
fn version_names_the_binary_and_its_version() {
    let out = sprig(&["--version"]);
    assert!(out.status.success());
    assert_eq!(
        stdout(&out).trim(),
        format!("sprig {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn help_lists_each_command_with_one_line_and_the_exit_codes() {
    let out = sprig(&["--help"]);
    assert!(out.status.success());
    let help = stdout(&out);
    for (command, words) in [
        ("next", "What can be done now"),
        ("check", "Report broken grafts"),
    ] {
        let line = help
            .lines()
            .find(|l| l.trim_start().starts_with(command))
            .unwrap_or_else(|| panic!("--help does not list {command}"));
        assert!(line.contains(words), "{line}");
    }
    for code in ["0  fine", "1  findings", "2  usage error"] {
        assert!(help.contains(code), "--help does not explain exit {code}");
    }
}

#[test]
fn no_command_is_a_usage_error() {
    let out = sprig(&[]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("Usage:"));
}

#[test]
fn an_unknown_argument_is_a_usage_error() {
    let out = sprig(&["--frobnicate"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("--frobnicate"));
}

// Commands removed before 1.0 (cairn 0154) say what replaced them, so a
// script or a habit that still uses one is told how to change, not just
// that it failed.
#[test]
fn a_removed_command_exits_2_and_names_what_replaced_it() {
    for (old, now) in [
        ("status", "sprig tree --summary"),
        ("agenda", "sprig find is:open"),
        ("mark", "sprig tick REF --mark MARK"),
        ("answer", "in your editor"),
        ("add", "in your editor"),
        ("edit", "your editor"),
        ("graph", "sprig export --format dot"),
        ("agent", "sprig mcp --instructions"),
        ("completions", "sprig generate SHELL"),
        ("man", "sprig generate man"),
    ] {
        let out = sprig(&[old, "x"]);
        assert_eq!(out.status.code(), Some(2), "{old}");
        let err = stderr(&out);
        assert!(
            err.contains(&format!("`sprig {old}` is gone")) && err.contains(now),
            "{old}: {err}"
        );
    }
    // An unknown word that was never a command gets clap's own message.
    assert!(!stderr(&sprig(&["frobnicate"])).contains("is gone"));
}

#[test]
fn an_unreadable_path_exits_2_and_names_the_path() {
    for command in ["next", "check"] {
        let out = sprig(&[command, "--today", "2026-10-01", "no/such/plan.sprig"]);
        assert_eq!(out.status.code(), Some(2), "{command}");
        assert!(
            stderr(&out).contains("no/such/plan.sprig"),
            "{}",
            stderr(&out)
        );
        assert!(stdout(&out).is_empty());
    }
}

#[test]
fn a_file_that_is_not_utf8_exits_2() {
    let dir = scratch("not-utf8");
    fs::write(dir.join("bad.sprig"), b"Plan\n- caf\xe9\n").unwrap();
    let out = sprig_in(&dir, &["check", "bad.sprig"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("bad.sprig"));
}

#[test]
fn every_command_takes_today_and_refuses_a_bad_one() {
    for command in ["next", "check"] {
        let out = sprig(&[command, "--today", "2026-10-01", "examples/bakery"]);
        assert_eq!(out.status.code(), Some(0), "{command}: {}", stderr(&out));
        let out = sprig(&["--today", "2026-10-01", command, "examples/bakery"]);
        assert_eq!(out.status.code(), Some(0), "{command}, global flag first");
        let out = sprig(&[command, "--today", "2026-02-30", "examples/bakery"]);
        assert_eq!(out.status.code(), Some(2), "{command}");
        assert!(stderr(&out).contains("2026-02-30"));
    }
}

#[test]
fn today_changes_what_has_started() {
    // Invite the street starts on 2026-11-16 (§7.5).
    let before = stdout(&sprig(&[
        "next",
        "--today",
        "2026-11-15",
        "examples/bakery",
    ]));
    let after = stdout(&sprig(&[
        "next",
        "--today",
        "2026-11-16",
        "examples/bakery",
    ]));
    assert!(!before.contains("Invite the street"));
    assert!(after.contains("Invite the street"), "{after}");
}

#[test]
fn next_for_the_bakery_matches_its_snapshot() {
    let out = sprig(&[
        "next",
        "--today",
        "2026-10-01",
        "examples/bakery/bakery.sprig",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    snapshot("next-bakery.txt", &stdout(&out));
    let out = sprig(&["next", "--today", "2026-10-01", "examples/bakery"]);
    assert!(out.status.success());
    snapshot("next-bakery-directory.txt", &stdout(&out));
}

#[test]
fn next_who_shows_only_that_owners_work() {
    let all = stdout(&sprig(&[
        "next",
        "--today",
        "2026-10-01",
        "examples/bakery",
    ]));
    let out = sprig(&[
        "next",
        "--today",
        "2026-10-01",
        "--who",
        "teo",
        "examples/bakery",
    ]);
    assert!(out.status.success());
    let teo = stdout(&out);
    snapshot("next-bakery-teo.txt", &teo);
    let items: Vec<&str> = teo.lines().take_while(|l| !l.is_empty()).collect();
    assert_eq!(items.len(), 3);
    for item in &items {
        // Columns are as wide as their widest cell, so compare the item alone.
        let text = item.split("  ").next().unwrap();
        assert!(all.contains(text), "{text} is not in the unfiltered list");
    }
    // Owned by dana in the bakery plan, so not teo's.
    assert!(!teo.contains("Break clause"));
    let at = sprig(&[
        "next",
        "--today",
        "2026-10-01",
        "--who",
        "@teo",
        "examples/bakery",
    ]);
    assert_eq!(stdout(&at), teo, "the @ is optional");
}

#[test]
fn next_counts_what_it_leaves_out_because_it_is_waiting() {
    let out = stdout(&sprig(&[
        "next",
        "--today",
        "2026-10-01",
        "examples/bakery/bakery.sprig",
    ]));
    assert!(out.contains("12 waiting on something, not shown"), "{out}");
    // Sign the lease waits on the terms and the permit.
    assert!(!out.contains("Sign the lease"));
    let limited = stdout(&sprig(&[
        "next",
        "--today",
        "2026-10-01",
        "--limit",
        "3",
        "examples/bakery/bakery.sprig",
    ]));
    assert_eq!(limited.lines().take_while(|l| !l.is_empty()).count(), 3);
    assert!(limited.contains("3 of 20 ready"));
}

#[test]
fn next_with_nothing_ready_exits_1() {
    let dir = scratch("nothing-ready");
    fs::write(
        dir.join("plan.sprig"),
        "Plan\n\nx Done\n- Later  start:2027-01-01\n",
    )
    .unwrap();
    let out = sprig_in(&dir, &["next", "--today", "2026-10-01"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stdout(&out).contains("Nothing is ready · 1 waiting on something"));
}

#[test]
fn check_passes_the_bakery() {
    let out = sprig(&["check", "examples/bakery"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(stdout(&out).is_empty());
    assert!(stderr(&out).contains("6 files checked, no problems"));
}

#[test]
fn check_fails_a_broken_link_and_names_the_file_and_line() {
    let dir = scratch("broken-link");
    copy_bakery(&dir);
    let kitchen = fs::read_to_string(dir.join("kitchen.sprig")).unwrap();
    let broken = kitchen.replace("after:[[lease^signed]]", "after:[[lease^signd]]");
    assert_ne!(kitchen, broken);
    fs::write(dir.join("kitchen.sprig"), broken).unwrap();

    let out = sprig_in(&dir, &["check", "."]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        stdout(&out),
        "kitchen.sprig:18:25: error[S0203]: no item ^signd in lease.sprig\n"
    );
    // Named one file at a time, it still finds what that file needs.
    let out = sprig_in(&dir, &["check", "kitchen.sprig"]);
    assert_eq!(out.status.code(), Some(1));
    let out = sprig_in(
        &dir.join(".."),
        &[
            "check",
            &format!(
                "{}/kitchen.sprig",
                dir.file_name().unwrap().to_string_lossy()
            ),
        ],
    );
    assert!(
        stdout(&out).contains("/kitchen.sprig:18:25: error[S0203]"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn check_lines_read_file_line_col_severity_and_code() {
    let dir = scratch("format");
    fs::write(
        dir.join("plan.sprig"),
        "Plan\n\n+ [[nowhere]]\n- Soon  due:soonish\n- Go  after:^gone\n",
    )
    .unwrap();
    let out = sprig_in(&dir, &["check"]);
    assert_eq!(out.status.code(), Some(1));
    let lines: Vec<String> = stdout(&out).lines().map(String::from).collect();
    assert_eq!(
        lines,
        [
            "plan.sprig:3:3: error[S0101]: no file named nowhere.sprig",
            "plan.sprig:4:9: warning[S0001]: due: \"soonish\" is not a date; it is kept as text",
            "plan.sprig:5:7: error[S0201]: no item ^gone in plan.sprig",
        ]
    );
    assert!(stderr(&out).contains("2 errors and 1 warning in 1 of 1 file"));
}

#[test]
fn strict_turns_warnings_into_a_failure() {
    let dir = scratch("strict");
    fs::write(dir.join("plan.sprig"), "Plan\n\n- Soon  due:soonish\n").unwrap();
    assert_eq!(sprig_in(&dir, &["check"]).status.code(), Some(0));
    assert_eq!(
        sprig_in(&dir, &["check", "--strict"]).status.code(),
        Some(1)
    );
}

#[test]
fn a_graft_out_of_the_workspace_is_refused_and_never_read() {
    let dir = scratch("escape");
    fs::create_dir_all(dir.join("plans")).unwrap();
    fs::write(dir.join("secret.sprig"), "Secret\n\n- the launch codes\n").unwrap();
    fs::write(
        dir.join("plans/plan.sprig"),
        "Plan\n\n+ [[../secret]]\n+ [[../../../../etc/passwd]]\n",
    )
    .unwrap();
    let out = sprig_in(&dir.join("plans"), &["check", "plan.sprig"]);
    assert_eq!(out.status.code(), Some(1));
    let text = stdout(&out);
    assert_eq!(text.matches("error[S0104]").count(), 2, "{text}");
    let out = sprig_in(
        &dir.join("plans"),
        &["next", "--today", "2026-10-01", "plan.sprig"],
    );
    assert!(!stdout(&out).contains("launch codes"));
}

#[cfg(unix)]
#[test]
fn a_symlink_out_of_the_workspace_is_not_followed() {
    let dir = scratch("symlink");
    fs::create_dir_all(dir.join("plans")).unwrap();
    fs::write(dir.join("secret.sprig"), "Secret\n\n- the launch codes\n").unwrap();
    std::os::unix::fs::symlink(dir.join("secret.sprig"), dir.join("plans/inside.sprig")).unwrap();
    fs::write(dir.join("plans/plan.sprig"), "Plan\n\n+ [[inside]]\n").unwrap();
    let out = sprig_in(&dir.join("plans"), &["next", "--today", "2026-10-01", "."]);
    assert!(!stdout(&out).contains("launch codes"), "{}", stdout(&out));
}

#[test]
fn a_deep_plan_does_not_overflow_the_stack() {
    let dir = scratch("deep");
    let mut src = String::from("Deep\n\n");
    for i in 0..3_000 {
        src.push_str(&" ".repeat(i));
        src.push_str("- step\n");
    }
    fs::write(dir.join("deep.sprig"), src).unwrap();
    let out = sprig_in(&dir, &["next", "--today", "2026-10-01"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(stdout(&out).contains("1 ready"));
}

#[test]
fn fmt_check_lists_files_and_fmt_rewrites_only_layout_and_dates() {
    let dir = scratch("fmt");
    fs::write(dir.join("tidy.sprig"), "Tidy\n\n- one\n  - two\n").unwrap();
    fs::write(
        dir.join("messy.sprig"),
        "Messy   \n\n- one  due:fri\n     - two\n\n\n",
    )
    .unwrap();
    let out = sprig_in(&dir, &["fmt", "--check", "--today", "2026-10-01"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stdout(&out), "messy.sprig\n");
    assert_eq!(
        fs::read_to_string(dir.join("messy.sprig")).unwrap(),
        "Messy   \n\n- one  due:fri\n     - two\n\n\n",
        "--check writes nothing"
    );

    let out = sprig_in(&dir, &["fmt", "--today", "2026-10-01"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(
        fs::read_to_string(dir.join("messy.sprig")).unwrap(),
        "Messy\n\n- one  due:2026-10-02\n  - two\n"
    );
    assert_eq!(
        fs::read_to_string(dir.join("tidy.sprig")).unwrap(),
        "Tidy\n\n- one\n  - two\n"
    );
    assert_eq!(
        sprig_in(&dir, &["fmt", "--check", "--today", "2026-10-01"])
            .status
            .code(),
        Some(0)
    );
}

#[test]
fn fmt_leaves_grafted_files_it_was_not_pointed_at_alone() {
    let dir = scratch("fmt-graft");
    fs::write(dir.join("a.sprig"), "A\n\n+ [[b]]\n").unwrap();
    fs::write(dir.join("b.sprig"), "B\n\n- x   \n").unwrap();
    let out = sprig_in(&dir, &["fmt", "--check", "a.sprig"]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
}

#[test]
fn the_repositorys_own_examples_are_formatted() {
    let out = sprig(&["fmt", "--check", "examples"]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
}

#[test]
fn tick_changes_only_the_mark_or_the_due_date() {
    let dir = scratch("tick");
    copy_bakery(&dir);
    let before = fs::read_to_string(dir.join("lease.sprig")).unwrap();
    let out = sprig_in(
        &dir,
        &[
            "tick",
            "--today",
            "2026-10-02",
            "lease^signed",
            "grease trap",
        ],
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "lease.sprig:13  x Sign the lease  ^signed  after:^terms  after:^permit\nlease.sprig:9  x Who pays for the grease trap?\n"
    );
    // What it unblocked is told to the person, beside the output.
    assert_eq!(
        stderr(&out),
        "No longer waiting on it:\n  \
         bakery.sprig:15  Supplier credit line  (unblocked)\n  \
         kitchen.sprig:18  Build  (still waits on Design)\n"
    );
    let after = fs::read_to_string(dir.join("lease.sprig")).unwrap();
    let diff: Vec<(usize, u8, u8)> = before
        .bytes()
        .zip(after.bytes())
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, (a, b))| (i, a, b))
        .collect();
    assert_eq!(before.len(), after.len());
    assert_eq!(diff.len(), 2, "exactly two marks changed: {diff:?}");
    assert!(diff.iter().all(|&(_, _, b)| b == b'x'));

    // A recurring item stays open and its due date moves on (§10.4).
    let bakery = fs::read_to_string(dir.join("bakery.sprig")).unwrap();
    let out = sprig_in(&dir, &["tick", "--today", "2026-10-02", "weekly numbers"]);
    assert!(
        stdout(&out).contains("- Weekly numbers to the bank  every:fri  due:2026-10-09"),
        "{}",
        stdout(&out)
    );
    let moved = fs::read_to_string(dir.join("bakery.sprig")).unwrap();
    assert_eq!(moved, bakery.replace("due:2026-10-02", "due:2026-10-09"));

    // Ticking a done item reopens it.
    sprig_in(&dir, &["tick", "lease.sprig:13"]);
    assert_eq!(
        fs::read_to_string(dir.join("lease.sprig"))
            .unwrap()
            .lines()
            .nth(12),
        Some("- Sign the lease  ^signed  after:^terms  after:^permit")
    );
}

#[test]
fn an_unknown_or_ambiguous_ref_ticks_nothing_and_lists_candidates() {
    let dir = scratch("tick-bad");
    copy_bakery(&dir);
    let before: Vec<String> = ["lease", "groceries"]
        .iter()
        .map(|f| fs::read_to_string(dir.join(format!("{f}.sprig"))).unwrap())
        .collect();
    let out = sprig_in(&dir, &["tick", "^signed", "sack"]);
    assert_eq!(out.status.code(), Some(2));
    let err = stderr(&out);
    assert!(
        err.contains("sack names 3 items:\n  groceries.sprig:4  Rye, 4 sacks"),
        "{err}"
    );
    let after: Vec<String> = ["lease", "groceries"]
        .iter()
        .map(|f| fs::read_to_string(dir.join(format!("{f}.sprig"))).unwrap())
        .collect();
    assert_eq!(before, after, "all or nothing");
}

#[test]
fn tick_reads_refs_from_standard_input() {
    use std::io::Write as _;
    let dir = scratch("tick-stdin");
    copy_bakery(&dir);
    let mut child = Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(["tick", "-"])
        .current_dir(&dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"^permit\nkitchen.sprig:13\n\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out).lines().count(), 2);
    assert!(
        fs::read_to_string(dir.join("kitchen.sprig"))
            .unwrap()
            .contains("  x Spiral mixer")
    );
}

#[test]
fn lsp_speaks_json_rpc_on_stdin_and_stdout_and_publishes_problems() {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::process::Stdio;

    let dir = scratch("lsp");
    copy_bakery(&dir);
    let path = dir
        .canonicalize()
        .unwrap()
        .display()
        .to_string()
        .replace('\\', "/");
    // Windows canonical paths start with \\?\, which a file URI does not.
    let path = path.trim_start_matches("//?/");
    let root = if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(["lsp", "--today", "2026-10-01"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the sprig binary runs");
    let mut stdin = child.stdin.take().unwrap();
    let mut send = move |body: String| {
        write!(stdin, "Content-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        stdin.flush().unwrap();
    };
    let mut reader = BufReader::new(child.stdout.take().unwrap());
    // One message as an editor reads it: headers, a blank line, the body.
    let mut next = move || {
        let mut len = 0;
        loop {
            let mut header = String::new();
            reader.read_line(&mut header).unwrap();
            let header = header.trim_end();
            if header.is_empty() {
                break;
            }
            if let Some(n) = header.strip_prefix("Content-Length: ") {
                len = n.parse().unwrap();
            }
        }
        let mut body = vec![0; len];
        reader.read_exact(&mut body).unwrap();
        String::from_utf8(body).unwrap()
    };

    send(format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"processId":null,"rootUri":"{root}","capabilities":{{}}}}}}"#
    ));
    let init = next();
    assert!(init.contains(r#""serverInfo":{"name":"sprig""#), "{init}");
    send(r#"{"jsonrpc":"2.0","method":"initialized","params":{}}"#.into());
    let broken =
        fs::read_to_string(dir.join("launch.sprig")).unwrap() + "- Posters  after:^nowhere\n";
    send(format!(
        r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"{root}/launch.sprig","languageId":"sprig","version":1,"text":{}}}}}}}"#,
        json_string(&broken)
    ));
    let published = next();
    assert!(
        published.contains("textDocument/publishDiagnostics"),
        "{published}"
    );
    assert!(published.contains(r#""code":"S0201""#), "{published}");
    send(r#"{"jsonrpc":"2.0","id":2,"method":"shutdown"}"#.into());
    assert!(next().contains(r#""id":2"#));
    send(r#"{"jsonrpc":"2.0","method":"exit"}"#.into());
    assert!(child.wait().unwrap().success());
}

/// A JSON string literal, for a test that writes JSON by hand.
fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
