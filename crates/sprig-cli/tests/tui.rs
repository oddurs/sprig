//! `sprig tui`, through `--screenshot` and `--keys`: the frames a person
//! sees, and the files a session of keys leaves behind.
//!
//! `SPRIG_BLESS=1 cargo test -p sprig-cli` rewrites the snapshots.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

fn repo(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(args)
        .current_dir(dir)
        .env("SPRIG_TODAY", "2026-10-01")
        .env_remove("VISUAL")
        .env_remove("EDITOR")
        .output()
        .expect("sprig runs")
}

fn stdout(o: &Output) -> String {
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stdout.clone()).unwrap()
}

fn shot(size: &str, keys: &str) -> String {
    let mut args = vec!["tui", "examples/bakery", "--screenshot", size];
    if !keys.is_empty() {
        args.extend(["--keys", keys]);
    }
    stdout(&run(&repo(""), &args))
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

/// A copy of the bakery, committed to a git repository of its own, so
/// `git diff` shows what a session changed.
fn bakery() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "tui-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for e in fs::read_dir(repo("examples/bakery")).unwrap() {
        let e = e.unwrap();
        fs::copy(e.path(), dir.join(e.file_name())).unwrap();
    }
    let git = |args: &[&str]| {
        let o = Command::new("git")
            .args(args)
            .current_dir(&dir)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.org")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.org")
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8(o.stdout).unwrap()
    };
    git(&["init", "-q"]);
    git(&["-c", "core.autocrlf=false", "add", "-A"]);
    git(&["commit", "-qm", "base"]);
    dir
}

fn diff(dir: &Path) -> Vec<String> {
    let o = Command::new("git")
        .args(["diff", "-U0", "--no-color"])
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8(o.stdout)
        .unwrap()
        .lines()
        .filter(|l| {
            (l.starts_with('+') || l.starts_with('-'))
                && !l.starts_with("+++")
                && !l.starts_with("---")
        })
        .map(String::from)
        .collect()
}

#[test]
fn the_tree_at_80_by_24_matches_its_snapshot() {
    snapshot("tui-tree-80x24.txt", &shot("80x24", ""));
}

#[test]
fn the_tree_at_120_by_40_matches_its_snapshot() {
    snapshot("tui-tree-120x40.txt", &shot("120x40", "jjjj"));
}

#[test]
fn below_80_columns_the_detail_pane_hides_and_nothing_overflows() {
    let frame = shot("60x20", "");
    snapshot("tui-tree-60x20.txt", &frame);
    assert!(!frame.contains('│'), "{frame}");
    for line in frame.lines() {
        assert!(
            unicode_width::UnicodeWidthStr::width(line) <= 60,
            "{line:?}"
        );
    }
    assert_eq!(frame.lines().count(), 20);
}

#[test]
fn next_up_and_the_agenda_match_their_snapshots() {
    snapshot("tui-next-100x30.txt", &shot("100x30", "2"));
    snapshot("tui-agenda-100x30.txt", &shot("100x30", "3"));
}

#[test]
fn the_help_overlay_and_a_filtered_tree_match_their_snapshots() {
    snapshot("tui-help-100x30.txt", &shot("100x30", "?"));
    // The matches keep their ancestors, so they stay in context.
    let filtered = shot("100x30", "/quote");
    snapshot("tui-filter-100x30.txt", &filtered);
    assert!(filtered.contains("Kitchen fit-out") && filtered.contains("Deck oven, three decks"));
    assert!(!filtered.contains("Business plan"));
}

#[test]
fn next_up_lists_what_sprig_next_does_in_the_same_order() {
    let plain = stdout(&run(&repo(""), &["next", "--plain", "examples/bakery"]));
    let labels: Vec<&str> = plain
        .lines()
        .map(|l| l.split('\t').nth(2).unwrap())
        .collect();
    let frame = shot("200x60", "2");
    let mut from = 0;
    for label in &labels {
        let at = frame[from..]
            .find(&format!(" {label}"))
            .unwrap_or_else(|| panic!("{label} is missing, or out of order:\n{frame}"));
        from += at + label.len();
    }
    // And nothing else: one row per item, each a cursor column, a space and
    // a mark.
    let rows = frame
        .lines()
        .filter(|l| {
            let c: Vec<char> = l.chars().take(4).collect();
            c.len() == 4 && "-~x?/>".contains(c[2]) && c[3] == ' '
        })
        .count();
    assert_eq!(rows, labels.len(), "{frame}");
}

/// Ticks, a mark and an add, each found with the filter.
const SESSION: &str = "/Sign the lease<Enter>Gx<Esc>\
/third quote<Enter>Gx<Esc>\
/Menu boards<Enter>G~aPrint the menus<Enter><Esc>\
/Weekly numbers<Enter>Gx<Esc>";

#[test]
fn a_session_of_keys_changes_exactly_the_lines_it_meant_to() {
    let dir = bakery();
    stdout(&run(
        &dir,
        &["tui", "--screenshot", "100x30", "--keys", SESSION],
    ));
    let mut got = diff(&dir);
    got.sort();
    let mut want = vec![
        "-- Sign the lease  ^signed  after:^terms  after:^permit",
        "+x Sign the lease  ^signed  after:^terms  after:^permit",
        "-    - Get a third quote  due:2026-10-02  est:2h",
        "+    x Get a third quote  due:2026-10-02  est:2h",
        "-  - Menu boards  after:^logo  est:4h",
        "+  ~ Menu boards  after:^logo  est:4h",
        "+  - Print the menus",
        // A recurring item moves to its next date, as `sprig tick` does.
        "-  - Weekly numbers to the bank  every:fri  due:2026-10-02",
        "+  - Weekly numbers to the bank  every:fri  due:2026-10-09",
    ];
    want.sort();
    assert_eq!(got, want);
}

#[test]
fn undoing_every_write_of_a_session_leaves_no_diff() {
    let dir = bakery();
    let keys = format!("{SESSION}uuuuu");
    let frame = stdout(&run(
        &dir,
        &["tui", "--screenshot", "100x30", "--keys", &keys],
    ));
    assert_eq!(diff(&dir), Vec::<String>::new());
    assert!(frame.contains("undone in"), "{frame}");
}

#[test]
fn outside_a_terminal_it_exits_2_and_points_at_screenshot() {
    let o = run(&repo(""), &["tui", "examples/bakery"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("--screenshot"));
}

#[test]
fn a_bad_screenshot_size_or_key_exits_2() {
    for args in [
        &["tui", "examples/bakery", "--screenshot", "wide"][..],
        &["tui", "examples/bakery", "--screenshot", "10x2"],
        &[
            "tui",
            "examples/bakery",
            "--screenshot",
            "80x24",
            "--keys",
            "<Nope>",
        ],
    ] {
        assert_eq!(run(&repo(""), args).status.code(), Some(2), "{args:?}");
    }
}

#[test]
fn the_frame_in_the_tui_guide_is_the_current_snapshot() {
    let page = fs::read_to_string(repo("site/src/content/docs/docs/tui.md")).unwrap();
    let frame = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/tui-tree-80x24.txt"),
    )
    .unwrap();
    assert!(
        page.contains(frame.replace("\r\n", "\n").trim_end()),
        "copy tui-tree-80x24.txt into the guide"
    );
}
