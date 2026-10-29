//! The `--plain` output of every command that has one, held to snapshots:
//! the columns, and their order, are a promise (docs/stability.md), so a
//! change to one of these files is a breaking change to the CLI.
//!
//! `SPRIG_BLESS=1 cargo test -p sprig-cli` rewrites the snapshots.

use std::fs;
use std::path::Path;
use std::process::Command;

fn plain(args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(args)
        .arg("--plain")
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .env("SPRIG_TODAY", "2026-10-01")
        .output()
        .unwrap();
    assert!(
        o.status.code().is_some_and(|c| c < 2),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8(o.stdout).unwrap()
}

fn snapshot(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name);
    if std::env::var_os("SPRIG_BLESS").is_some() {
        fs::write(&path, actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("no snapshot {name}; run with SPRIG_BLESS=1"));
    assert_eq!(actual, expected.replace("\r\n", "\n"), "snapshot {name}");
}

/// Every line has the same number of fields, so a script can rely on them.
fn columns(out: &str, n: usize) {
    for l in out.lines() {
        assert_eq!(l.split('\t').count(), n, "{l:?}");
    }
}

#[test]
fn the_plain_columns_of_next_stay_as_they_are() {
    let out = plain(&["next", "examples/bakery"]);
    columns(&out, 6);
    snapshot("plain-next.txt", &out);
}

#[test]
fn the_plain_columns_of_check_stay_as_they_are() {
    let out = plain(&["check", "tests/conformance/graph"]);
    columns(&out, 6);
    // The message is the last column and may be reworded; its place is the
    // promise, not its words.
    let out: String = out
        .lines()
        .map(|l| format!("{}\t…\n", l.rsplit_once('\t').unwrap().0))
        .collect();
    snapshot("plain-check.txt", &out);
}

#[test]
fn the_plain_columns_of_tree_stay_as_they_are() {
    let out = plain(&["tree", "examples/bakery/kitchen.sprig"]);
    columns(&out, 7);
    snapshot("plain-tree.txt", &out);
}

#[test]
fn the_plain_columns_of_status_stay_as_they_are() {
    let out = plain(&["status", "examples/bakery"]);
    columns(&out, 7);
    snapshot("plain-status.txt", &out);
}

#[test]
fn the_plain_columns_of_find_stay_as_they_are() {
    let out = plain(&["find", "--in", "examples/bakery", "is:open"]);
    columns(&out, 6);
    snapshot("plain-find.txt", &out);
}

#[test]
fn the_plain_columns_of_agenda_stay_as_they_are() {
    let out = plain(&["agenda", "examples/bakery"]);
    columns(&out, 5);
    snapshot("plain-agenda.txt", &out);
}
