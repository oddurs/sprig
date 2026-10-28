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
            "plan.sprig:4:9: warning[S0001]: due:soonish is not a date; it is kept as text",
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
