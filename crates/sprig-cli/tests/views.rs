//! The read-only views and the output layer: `tree`, `why`, `status`,
//! `--plain`, colour and widths, and what `generate` prints: completions and
//! the manual page.
//!
//! `SPRIG_BLESS=1 cargo test -p sprig-cli` rewrites the snapshots.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

fn repo(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_sprig"));
    c.args(args)
        .current_dir(dir)
        .env_remove("NO_COLOR")
        .env_remove("COLUMNS")
        .env("SPRIG_TODAY", "2026-10-01");
    for (k, v) in env {
        c.env(k, v);
    }
    c.output().expect("sprig runs")
}

fn sprig(args: &[&str]) -> Output {
    run(&repo(""), args, &[])
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

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

// ---------------------------------------------------------------------------
// The output layer

#[test]
fn no_color_and_color_never_print_the_same_bytes_with_no_escapes() {
    for args in [
        &["next", "examples/bakery"][..],
        &["tree", "examples/bakery/kitchen.sprig"],
        &["tree", "--summary", "examples/bakery"],
    ] {
        let never = stdout(&run(
            &repo(""),
            &[args, &["--color", "never"]].concat(),
            &[],
        ));
        let no_color = stdout(&run(&repo(""), args, &[("NO_COLOR", "1")]));
        assert_eq!(never, no_color, "{args:?}");
        assert!(!never.contains('\x1b'), "{args:?}");
        // And `always` does colour the marks, which is all it colours.
        let always = stdout(&run(
            &repo(""),
            &[args, &["--color", "always"]].concat(),
            &[],
        ));
        assert!(always.contains("\x1b[38;5;"), "{args:?}");
    }
    // An explicit flag beats NO_COLOR, as no-color.org asks.
    let forced = stdout(&run(
        &repo(""),
        &["next", "examples/bakery", "--color", "always"],
        &[("NO_COLOR", "1")],
    ));
    assert!(forced.contains('\x1b'));
}

#[test]
fn widths_truncate_with_an_ellipsis_and_never_mid_character() {
    for cols in [60usize, 120] {
        let out = stdout(&run(
            &repo(""),
            &["next", "examples/bakery"],
            &[("COLUMNS", &cols.to_string())],
        ));
        let rows: Vec<&str> = out.lines().take_while(|l| !l.is_empty()).collect();
        assert!(!rows.is_empty());
        for line in &rows {
            let w = unicode_width::UnicodeWidthStr::width(*line);
            assert!(w <= cols, "{cols}: {w} columns: {line}");
        }
        if cols == 60 {
            assert!(
                rows.iter().any(|l| l.contains('…')),
                "something had to give at 60"
            );
        }
    }
    // Wide characters are cut whole.
    let dir = scratch("wide");
    fs::write(
        dir.join("plan.sprig"),
        "計画\n\n- 日本語のとても長いタスクの名前をここに書きます  due:2026-10-03\n",
    )
    .unwrap();
    let out = stdout(&run(&dir, &["next"], &[("COLUMNS", "30")]));
    let line = out.lines().next().unwrap();
    assert!(unicode_width::UnicodeWidthStr::width(line) <= 30, "{line}");
    assert!(line.contains('…'));
}

#[test]
fn plain_next_feeds_tick_through_cut() {
    let dir = scratch("pipe");
    for e in fs::read_dir(repo("examples/bakery")).unwrap() {
        let p = e.unwrap().path();
        fs::copy(&p, dir.join(p.file_name().unwrap())).unwrap();
    }
    let plain = stdout(&run(&dir, &["next", "--plain", "--limit", "3"], &[]));
    let lines: Vec<&str> = plain.lines().collect();
    assert_eq!(lines.len(), 3);
    for l in &lines {
        assert_eq!(l.split('\t').count(), 6, "{l}");
        assert!(!l.contains('\x1b'));
    }
    // `cut -f1`, done in Rust so the test runs everywhere.
    let refs: String = lines
        .iter()
        .map(|l| format!("{}\n", l.split('\t').next().unwrap()))
        .collect();
    let mut child = Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(["tick", "-"])
        .current_dir(&dir)
        .env("SPRIG_TODAY", "2026-10-01")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write as _;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(refs.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(stdout(&out).lines().count(), 3);
    // They are done now, so next no longer lists them.
    let after = stdout(&run(&dir, &["next", "--plain"], &[]));
    for r in refs.lines() {
        assert!(
            !after.lines().any(|l| l.starts_with(&format!("{r}\t"))),
            "{r} is still ready"
        );
    }
}

#[test]
fn plain_check_is_one_tab_separated_problem_per_line() {
    let dir = scratch("plain-check");
    fs::write(dir.join("p.sprig"), "P\n\n+ [[nowhere]]\n").unwrap();
    let out = stdout(&run(&dir, &["check", "--plain"], &[]));
    assert_eq!(
        out,
        "p.sprig\t3\t3\terror\tS0101\tno file named nowhere.sprig\n"
    );
}

// ---------------------------------------------------------------------------
// tree

#[test]
fn tree_for_the_bakery_matches_its_snapshot_with_grafts_under_their_lines() {
    let out = sprig(&["tree", "examples/bakery/bakery.sprig", "--color", "never"]);
    assert_eq!(out.status.code(), Some(0));
    let text = stdout(&out);
    snapshot("tree-bakery.txt", &text);
    // Kitchen's items sit under its graft line.
    let graft = text
        .lines()
        .position(|l| l.contains("Kitchen fit-out"))
        .unwrap();
    let oven = text.lines().position(|l| l.contains("Deck oven")).unwrap();
    assert!(oven > graft);
}

#[test]
fn tree_depth_and_hide_done_trim_it() {
    let full = stdout(&sprig(&["tree", "examples/bakery/lease.sprig"]));
    let shallow = stdout(&sprig(&[
        "tree",
        "examples/bakery/lease.sprig",
        "--depth",
        "1",
    ]));
    assert!(full.contains("Break clause") && !shallow.contains("Break clause"));
    let open = stdout(&sprig(&[
        "tree",
        "examples/bakery/lease.sprig",
        "--hide-done",
    ]));
    assert!(full.contains("Shortlist") && !open.contains("Shortlist"));
}

// ---------------------------------------------------------------------------
// why

#[test]
fn why_tiling_crosses_into_the_lease_and_exits_1() {
    let out = sprig(&["why", "--in", "examples/bakery", "Tiling"]);
    assert_eq!(out.status.code(), Some(1));
    let text = stdout(&out);
    snapshot("why-tiling.txt", &text);
    assert!(text.contains("kitchen.sprig:22") && text.contains("lease.sprig:13"));
}

#[test]
fn why_on_an_item_that_is_not_blocked_says_so_and_exits_0() {
    let out = sprig(&["why", "--in", "examples/bakery", "^permit"]);
    assert_eq!(out.status.code(), Some(0));
    let text = stdout(&out);
    assert_eq!(text.lines().count(), 2, "{text}");
    assert!(text.contains("not blocked: ready to start"));
}

#[test]
fn a_cycle_is_printed_once_and_marked() {
    let dir = scratch("cycle");
    fs::write(
        dir.join("p.sprig"),
        "P\n\n- A  ^a  after:^b\n- B  ^b  after:^a\n",
    )
    .unwrap();
    let out = run(&dir, &["why", "^a"], &[]);
    assert_eq!(out.status.code(), Some(1));
    let text = stdout(&out);
    assert_eq!(text.matches("a cycle").count(), 1, "{text}");
    assert!(text.lines().count() < 6, "{text}");
}

// ---------------------------------------------------------------------------
// status

#[test]
fn the_summary_of_the_bakery_matches_its_snapshot() {
    let out = sprig(&["tree", "--summary", "examples/bakery", "--color", "never"]);
    assert_eq!(out.status.code(), Some(0));
    snapshot("summary-bakery.txt", &stdout(&out));
}

#[test]
fn status_oneline_is_short_and_exits_1_when_something_is_late() {
    let on_time = sprig(&["tree", "--summary", "--oneline", "examples/bakery"]);
    assert_eq!(on_time.status.code(), Some(0));
    assert_eq!(stdout(&on_time), "sprig 21% · 21 ready · 12 waiting\n");
    let late = run(
        &repo(""),
        &["tree", "--summary", "--oneline", "examples/bakery"],
        &[("SPRIG_TODAY", "2026-11-20")],
    );
    assert_eq!(late.status.code(), Some(1));
    let line = stdout(&late);
    assert!(line.contains("late"), "{line}");
    assert!(line.trim_end().chars().count() <= 60);
}

#[test]
fn a_file_with_errors_still_gets_its_line() {
    let dir = scratch("status-errors");
    fs::write(dir.join("a.sprig"), "A\n\n- one\n").unwrap();
    fs::write(
        dir.join("b.sprig"),
        "B\n\n+ [[nowhere]]\n+ [[gone]]\n- two\n",
    )
    .unwrap();
    let out = stdout(&run(&dir, &["tree", "--summary", "--color", "never"], &[]));
    let b = out
        .lines()
        .find(|l| l.starts_with('b'))
        .unwrap_or_else(|| panic!("{out}"));
    assert!(b.contains("2 errors"), "{b}");
    assert!(out.lines().any(|l| l.starts_with('a')));
}

// ---------------------------------------------------------------------------
// sprig generate: completions and the manual

#[test]
fn every_shells_script_is_generated() {
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let out = sprig(&["generate", shell]);
        assert_eq!(out.status.code(), Some(0), "{shell}");
        assert!(stdout(&out).contains("tick"), "{shell}");
    }
}

#[test]
fn refs_complete_from_the_folder_in_bash_and_zsh() {
    let bash = stdout(&sprig(&["generate", "bash"]));
    assert_eq!(
        bash.matches("$(sprig __complete refs 2>/dev/null)").count(),
        2,
        "tick and why"
    );
    let zsh = stdout(&sprig(&["generate", "zsh"]));
    assert_eq!(zsh.matches(":_sprig_refs'").count(), 2, "tick and why");
    assert!(zsh.contains("_sprig_refs() {"));
}

#[test]
fn the_bash_script_loads_and_offers_the_folders_anchors() {
    // On Windows `bash` may be WSL's launcher, which says nothing about the
    // script; the script is still generated above.
    if cfg!(windows) || Command::new("bash").arg("--version").output().is_err() {
        eprintln!("no usable bash here; the bash script was generated but not loaded");
        return;
    }
    let script = scratch("bash").join("sprig.bash");
    fs::write(&script, stdout(&sprig(&["generate", "bash"]))).unwrap();
    let bin = Path::new(env!("CARGO_BIN_EXE_sprig"))
        .parent()
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/");
    let probe = format!(
        "export PATH=\"{bin}:$PATH\"; source '{}' || exit 1; COMP_WORDS=(sprig tick '^'); COMP_CWORD=2; _sprig sprig '^' tick; printf '%s\\n' \"${{COMPREPLY[@]}}\"",
        script.to_string_lossy().replace('\\', "/")
    );
    let out = Command::new("bash")
        .args(["-c", &probe])
        .current_dir(repo("examples/bakery"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let offered = stdout(&out);
    for a in ["^signed", "^oven", "^loan"] {
        assert!(
            offered.lines().any(|l| l == a),
            "bash does not offer {a}: {offered}"
        );
    }
}

#[test]
fn the_zsh_script_loads_where_zsh_is_installed() {
    if Command::new("zsh").arg("--version").output().is_err() {
        eprintln!("zsh is not installed here; the zsh script was generated but not loaded");
        return;
    }
    let script = scratch("zsh").join("_sprig");
    fs::write(&script, stdout(&sprig(&["generate", "zsh"]))).unwrap();
    let probe = format!(
        "autoload -U compinit && compinit -u && source '{}'",
        script.display()
    );
    let out = Command::new("zsh").args(["-c", &probe]).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn pressing_tab_in_zsh_offers_the_folders_anchors() {
    if Command::new("zsh").arg("--version").output().is_err() {
        eprintln!("zsh is not installed here; Tab completion was not tried");
        return;
    }
    // An interactive zsh under zpty, the way a person types: the script is
    // autoloaded from fpath, as a package manager installs it.
    let dir = scratch("zsh-tab");
    fs::write(dir.join("_sprig"), stdout(&sprig(&["generate", "zsh"]))).unwrap();
    let bin = Path::new(env!("CARGO_BIN_EXE_sprig")).parent().unwrap();
    let out = Command::new("zsh")
        .arg("-f")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/zsh-tab.zsh"))
        .arg(&dir)
        .arg(bin)
        .arg(repo("examples/bakery"))
        .arg("sprig tick ^")
        .args(["signed", "terms", "loan", "oven"])
        .output()
        .unwrap();
    let screen = String::from_utf8_lossy(&out.stdout);
    assert!(!screen.contains("command not found"), "{screen}");
    for a in ["^signed", "^terms", "^loan", "^oven"] {
        assert!(screen.contains(a), "zsh does not offer {a}:\n{screen}");
    }
}

#[test]
fn completion_candidates_are_the_folders_anchors_and_people() {
    let refs = stdout(&run(&repo("examples/bakery"), &["__complete", "refs"], &[]));
    assert!(refs.lines().any(|l| l == "^signed"));
    let people = stdout(&run(
        &repo("examples/bakery"),
        &["__complete", "people"],
        &[],
    ));
    assert_eq!(people, "dana\nkim\nteo\n");
}

#[test]
fn the_manual_covers_every_command_and_the_exit_codes() {
    let page = stdout(&sprig(&["generate", "man"]));
    assert!(
        page.starts_with(".ie") || page.starts_with(".TH"),
        "{}",
        &page[..40.min(page.len())]
    );
    for cmd in [
        "NEXT", "CHECK", "FMT", "TICK", "TREE", "WHY", "FIND", "GENERATE",
    ] {
        assert!(
            page.contains(&format!(".SH \"SPRIG {cmd}\"")),
            "no section for {cmd}"
        );
    }
    assert!(page.contains("Exit codes"));
    assert!(!page.contains("__complete"));
    // Rendered as `man -l` would, where groff is installed.
    let Ok(mut groff) = Command::new("groff")
        .args(["-man", "-Tutf8", "-ww", "-z"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    else {
        eprintln!("groff is not installed here; the page was generated but not rendered");
        return;
    };
    use std::io::Write as _;
    groff
        .stdin
        .take()
        .unwrap()
        .write_all(page.as_bytes())
        .unwrap();
    let out = groff.wait_with_output().unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stderr), "", "groff warned");
}
