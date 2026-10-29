//! The v0.3 tools: `next --json`, `find`, `agenda`, `mark`, `answer`, `add`,
//! `edit`, `graph`, `export` and `import`.
//!
//! `SPRIG_BLESS=1 cargo test -p sprig-cli` rewrites the snapshots.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

fn repo(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .env_remove("COLUMNS")
        .env("SPRIG_TODAY", "2026-10-01")
        .stdin(Stdio::null())
        .output()
        .expect("sprig runs")
}

fn sprig(args: &[&str]) -> Output {
    run(&repo(""), args)
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
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

/// A copy of the bakery to write to.
fn bakery() -> PathBuf {
    let dir = scratch("bakery");
    for e in fs::read_dir(repo("examples/bakery")).unwrap() {
        let e = e.unwrap();
        fs::copy(e.path(), dir.join(e.file_name())).unwrap();
    }
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

fn schema(def: Option<&str>) -> jsonschema::Validator {
    let mut s: Value =
        serde_json::from_str(&fs::read_to_string(repo("spec/tree.schema.json")).unwrap()).unwrap();
    if let Some(def) = def {
        // The same definitions, with one of them as the root.
        let defs = s["$defs"].clone();
        s = serde_json::json!({ "$ref": format!("#/$defs/{def}"), "$defs": defs });
    }
    jsonschema::validator_for(&s).expect("the tree schema is valid JSON Schema")
}

fn valid(v: &jsonschema::Validator, doc: &Value) {
    let errors: Vec<String> = v.iter_errors(doc).map(|e| e.to_string()).collect();
    assert!(errors.is_empty(), "{errors:#?}\n{doc:#}");
}

fn json(o: &Output) -> Value {
    serde_json::from_str(&stdout(o)).unwrap_or_else(|e| panic!("{e}:\n{}", stdout(o)))
}

/// The refs in the first column of `--plain` output.
fn refs(plain: &str) -> Vec<String> {
    plain
        .lines()
        .map(|l| l.split('\t').next().unwrap().to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// next --json

#[test]
fn next_json_gives_the_items_next_shows_in_the_same_order_and_valid() {
    let j = json(&sprig(&["next", "--json", "examples/bakery"]));
    let items = j.as_array().unwrap();
    assert!(!items.is_empty());
    let node = schema(Some("resolved"));
    for i in items {
        valid(&node, &i["node"]);
    }
    let from_json: Vec<String> = items
        .iter()
        .map(|i| i["ref"].as_str().unwrap().to_string())
        .collect();
    let text = stdout(&sprig(&["next", "--plain", "examples/bakery"]));
    assert_eq!(from_json, refs(&text));
}

// ---------------------------------------------------------------------------
// find

fn found(query: &[&str]) -> Vec<String> {
    let o = sprig(&[&["find", "--plain", "--in", "examples/bakery"], query].concat());
    assert!(
        matches!(o.status.code(), Some(0 | 1)),
        "{query:?}: {}",
        stderr(&o)
    );
    refs(&stdout(&o))
}

/// The labels of what a query finds.
fn found_labels(query: &[&str]) -> Vec<String> {
    let o = sprig(&[&["find", "--plain", "--in", "examples/bakery"], query].concat());
    stdout(&o)
        .lines()
        .map(|l| l.split('\t').nth(2).unwrap().to_string())
        .collect()
}

#[test]
fn find_by_person_names_only_their_items() {
    let got = found_labels(&["@kim"]);
    assert!(got.contains(&"Logo, second round".to_string()), "{got:?}");
    assert!(!got.contains(&"Bank meeting".to_string()), "{got:?}");
}

#[test]
fn find_by_tag_finds_the_lines_that_write_it() {
    // Tags, unlike people, do not reach down (§4.3, §6.1).
    assert_eq!(found_labels(&["#finance"]), ["Money", "Equipment"]);
}

#[test]
fn find_by_status_follows_the_computed_status() {
    let doing = found_labels(&["is:doing"]);
    assert!(
        doing.contains(&"Logo, second round".to_string()),
        "{doing:?}"
    );
    let done = found_labels(&["is:done"]);
    assert!(done.contains(&"Bank meeting".to_string()), "{done:?}");
    assert!(found_labels(&["is:ask"]).contains(&"Kiln & Crumb, or Mill Street Bread?".to_string()));
    let open = found_labels(&["is:open"]);
    assert!(open.contains(&"Logo, second round".to_string()));
    assert!(!open.contains(&"Bank meeting".to_string()));
    let todo = found_labels(&["is:todo"]);
    assert!(todo.contains(&"Menu boards".to_string()));
    assert!(!todo.contains(&"Logo, second round".to_string()));
    assert_eq!(found_labels(&["is:dropped"]), ["Proofing cabinet"]);
    assert_eq!(found_labels(&["is:later"]), ["Espresso machine"]);
    // Nothing found is not an error, but a script can tell.
    let o = sprig(&["find", "--in", "examples/bakery", "is:done", "@nobody"]);
    assert_eq!(o.status.code(), Some(1));
}

#[test]
fn find_blocked_finds_what_waits_and_json_says_on_what() {
    let got = found_labels(&["is:blocked"]);
    assert!(got.contains(&"Window lettering".to_string()), "{got:?}");
    assert!(!got.contains(&"Logo, second round".to_string()), "{got:?}");

    let j = json(&sprig(&[
        "find",
        "--json",
        "--in",
        "examples/bakery",
        "is:blocked",
    ]));
    let node = schema(Some("resolved"));
    let items = j.as_array().unwrap();
    for i in items {
        valid(&node, &i["node"]);
    }
    let window = items
        .iter()
        .find(|i| i["node"]["label"] == "Window lettering")
        .unwrap();
    let waits: Vec<&str> = window["node"]["waits_on"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["label"].as_str().unwrap())
        .collect();
    assert_eq!(
        waits,
        ["Logo, second round", "Kiln & Crumb, or Mill Street Bread?"]
    );
}

#[test]
fn find_by_due_date_compares_against_the_date_given() {
    let before = found_labels(&["due:<2026-10-05"]);
    assert!(
        before.contains(&"Get a third quote".to_string()),
        "{before:?}"
    );
    assert!(
        !before.contains(&"Negotiate terms".to_string()),
        "{before:?}"
    );
    assert!(found_labels(&["due:<=2026-10-05"]).contains(&"Negotiate terms".to_string()));
    let after = found_labels(&["due:>2026-10-05"]);
    assert!(after.contains(&"Logo, second round".to_string()));
    assert!(!after.contains(&"Negotiate terms".to_string()));
    assert!(found_labels(&["due:>=2026-10-05"]).contains(&"Negotiate terms".to_string()));
    assert_eq!(found_labels(&["due:2026-10-05"]), ["Negotiate terms"]);
    // Typed dates read against today, as they do in a file.
    assert_eq!(found(&["due:fri"]), found(&["due:2026-10-02"]));
}

#[test]
fn find_by_words_and_filters_together_needs_all_of_them() {
    assert_eq!(found_labels(&["quote"]), ["Get a third quote"]);
    assert_eq!(found_labels(&["QUOTE", "@teo"]), ["Get a third quote"]);
    assert!(found_labels(&["quote", "@kim"]).is_empty());
}

#[test]
fn find_with_an_unknown_filter_exits_2_and_lists_the_filters() {
    let o = sprig(&["find", "--in", "examples/bakery", "colour:red"]);
    assert_eq!(o.status.code(), Some(2));
    let e = stderr(&o);
    for f in ["@who", "#tag", "is:", "blocked", "due:<DATE"] {
        assert!(e.contains(f), "{e}");
    }
}

// ---------------------------------------------------------------------------
// agenda and export

#[test]
fn agenda_for_the_bakery_matches_its_snapshot() {
    snapshot(
        "agenda-bakery.txt",
        &stdout(&sprig(&["agenda", "examples/bakery"])),
    );
}

#[test]
fn a_recurring_item_appears_once_on_its_next_date() {
    let out = stdout(&sprig(&[
        "agenda",
        "--days",
        "60",
        "--plain",
        "examples/bakery",
    ]));
    let weekly: Vec<&str> = out
        .lines()
        .filter(|l| l.contains("Weekly numbers"))
        .collect();
    assert_eq!(weekly.len(), 1, "{out}");
    assert!(weekly[0].contains("2026-10-02"), "{out}");
}

/// The events of an export, checked against RFC 5545 as they are read.
fn events(ics: &str) -> Vec<icalendar::Event> {
    use icalendar::Component;
    // What the parser forgives: line endings and folding (§3.1).
    for line in ics.split_inclusive('\n') {
        assert!(line.ends_with("\r\n"), "a line not ended by CRLF: {line:?}");
        assert!(line.len() <= 77, "a line over 75 octets: {line:?}");
    }
    let cal: icalendar::Calendar = ics.parse().expect("the export parses as iCalendar");
    for p in ["VERSION", "PRODID"] {
        assert!(cal.property_value(p).is_some(), "no {p}");
    }
    let events: Vec<icalendar::Event> = cal
        .components
        .into_iter()
        .filter_map(|c| match c {
            icalendar::CalendarComponent::Event(e) => Some(e),
            _ => None,
        })
        .collect();
    let mut uids = BTreeSet::new();
    for e in &events {
        // Required in every VEVENT (§3.6.1), with a DTSTART since no METHOD.
        let uid = e.get_uid().expect("every event has a UID");
        assert!(uids.insert(uid.to_string()), "UID {uid} is used twice");
        assert!(
            e.property_value("DTSTAMP").is_some(),
            "{uid} has no DTSTAMP"
        );
        assert!(e.get_start().is_some(), "{uid} has no DTSTART");
        assert!(e.get_summary().is_some(), "{uid} has no SUMMARY");
    }
    events
}

fn date_of(e: &icalendar::Event) -> String {
    use icalendar::{Component, DatePerhapsTime};
    match e.get_start().unwrap() {
        DatePerhapsTime::Date(d) => d.format("%Y-%m-%d").to_string(),
        other => panic!("a timed start: {other:?}"),
    }
}

#[test]
fn the_ics_export_is_valid_icalendar() {
    let ics = stdout(&sprig(&["export", "--format", "ics", "examples/bakery"]));
    assert!(!events(&ics).is_empty());
}

#[test]
fn the_agenda_and_the_ics_export_list_the_same_items_for_a_window() {
    use icalendar::Component;
    let agenda = stdout(&sprig(&["agenda", "--plain", "examples/bakery"]));
    let shown: BTreeSet<(String, String)> = agenda
        .lines()
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            (c[1].to_string(), c[3].to_string())
        })
        .collect();
    let ics = stdout(&sprig(&["export", "--format", "ics", "examples/bakery"]));
    // The agenda's default window: today and the thirteen days after.
    let exported: BTreeSet<(String, String)> = events(&ics)
        .iter()
        .map(|e| (date_of(e), e.get_summary().unwrap().to_string()))
        .filter(|(d, _)| d.as_str() <= "2026-10-14")
        .collect();
    assert_eq!(shown, exported);
}

#[test]
fn renaming_an_anchored_item_keeps_its_uid() {
    use icalendar::Component;
    let dir = bakery();
    let uid_of = |label: &str| {
        let ics = stdout(&run(&dir, &["export", "--format", "ics"]));
        events(&ics)
            .iter()
            .find(|e| e.get_summary() == Some(label))
            .map(|e| e.get_uid().unwrap().to_string())
            .unwrap_or_else(|| panic!("no event {label}"))
    };
    let before = uid_of("Equipment loan paperwork");
    let path = dir.join("bakery.sprig");
    let src = fs::read_to_string(&path).unwrap();
    fs::write(&path, src.replace("Equipment loan paperwork", "Loan forms")).unwrap();
    assert_eq!(uid_of("Loan forms"), before);
}

#[test]
fn the_json_export_is_a_list_of_tree_documents() {
    let j = json(&sprig(&["export", "--format", "json", "examples/bakery"]));
    let docs = j.as_array().unwrap();
    // One per tree: lease, kitchen and launch are grafted into the bakery's.
    let files: Vec<&str> = docs.iter().map(|d| d["file"].as_str().unwrap()).collect();
    assert_eq!(files, ["bakery.sprig", "dana.sprig"]);
    let tree = schema(None);
    for d in docs {
        valid(&tree, d);
    }
}

#[test]
fn the_csv_export_has_a_row_per_line_that_counts() {
    let csv = stdout(&sprig(&["export", "--format", "csv", "examples/bakery"]));
    let mut lines = csv.lines();
    assert_eq!(
        lines.next(),
        Some("file,line,mark,text,status,people,due,progress,path")
    );
    // A label with a comma is quoted, so a spreadsheet keeps it in one cell.
    assert!(csv.contains("\"Deck oven, three decks\""), "{csv}");
}

// ---------------------------------------------------------------------------
// graph

#[test]
fn graph_ids_are_stable_and_named_after_anchors_or_lines() {
    let one = stdout(&sprig(&["graph", "examples/bakery"]));
    assert_eq!(one, stdout(&sprig(&["graph", "examples/bakery"])));
    assert!(one.contains("bakery_loan ["), "{one}");
    // An edge runs from what is waited on to what waits.
    assert!(one.contains("launch_logo -> launch_l6"), "{one}");
    let m = stdout(&sprig(&["graph", "--format", "mermaid", "examples/bakery"]));
    assert!(m.starts_with("flowchart LR\n"), "{m}");
    assert!(m.contains("bakery_loan["), "{m}");
}

#[test]
fn graphviz_draws_the_bakery() {
    if Command::new("dot").arg("-V").output().is_err() {
        // CI sets this where it installs Graphviz, so the test cannot pass
        // there by skipping.
        assert!(
            std::env::var_os("SPRIG_CI_TOOLS").is_none_or(|v| v.is_empty()),
            "SPRIG_CI_TOOLS is set but dot is not installed"
        );
        eprintln!("Graphviz is not installed here; the graph was not drawn");
        return;
    }
    let mut dot = Command::new("dot")
        .arg("-Tsvg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    dot.stdin
        .take()
        .unwrap()
        .write_all(&sprig(&["graph", "examples/bakery"]).stdout)
        .unwrap();
    let o = dot.wait_with_output().unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(stdout(&o).contains("<svg"), "{}", stdout(&o));
}

// ---------------------------------------------------------------------------
// mark and answer

#[test]
fn answering_the_name_frees_window_lettering_of_it() {
    let dir = bakery();
    let o = run(&dir, &["answer", "^name", "Kiln & Crumb"]);
    assert!(o.status.success(), "{}", stderr(&o));
    snapshot("answer-name.txt", &stdout(&o));
}

#[test]
fn marking_the_lease_signed_lists_what_it_unblocks() {
    let dir = bakery();
    let o = run(&dir, &["mark", "^signed", "x"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let out = stdout(&o);
    assert!(out.starts_with("lease.sprig:13  x Sign the lease"), "{out}");
    assert!(
        out.contains("bakery.sprig:15  Supplier credit line  (unblocked)"),
        "{out}"
    );
}

#[test]
fn marking_a_group_or_a_graft_exits_2_and_writes_nothing() {
    let dir = bakery();
    for r in ["launch.sprig:3", "bakery.sprig:18"] {
        let before = fs::read_to_string(dir.join(r.split(':').next().unwrap())).unwrap();
        let o = run(&dir, &["mark", r, "x"]);
        assert_eq!(o.status.code(), Some(2), "{r}");
        assert!(
            stderr(&o).contains("status comes from its children"),
            "{}",
            stderr(&o)
        );
        let after = fs::read_to_string(dir.join(r.split(':').next().unwrap())).unwrap();
        assert_eq!(before, after);
    }
}

#[test]
fn mark_takes_the_mark_or_its_name() {
    let dir = bakery();
    for (m, want) in [
        ("~", "~ Menu boards"),
        ("later", "> Menu boards"),
        ("-", "- Menu boards"),
    ] {
        let o = run(&dir, &["mark", "launch.sprig:7", m]);
        assert!(o.status.success(), "{m}: {}", stderr(&o));
        assert!(stdout(&o).contains(want), "{m}: {}", stdout(&o));
    }
}

// ---------------------------------------------------------------------------
// add

#[test]
fn add_matches_the_indentation_of_the_siblings_it_joins() {
    for indent in ["\t", "    "] {
        let dir = scratch("indent");
        let src = format!("Plan\n\n- Parent  ^p\n{indent}x First\n- Other\n");
        fs::write(dir.join("plan.sprig"), &src).unwrap();
        let o = run(&dir, &["add", "Second", "--under", "^p"]);
        assert!(o.status.success(), "{}", stderr(&o));
        assert_eq!(
            fs::read_to_string(dir.join("plan.sprig")).unwrap(),
            format!("Plan\n\n- Parent  ^p\n{indent}x First\n{indent}- Second\n- Other\n"),
            "{indent:?}"
        );
    }
}

#[test]
fn add_writes_typed_dates_out_and_prints_where() {
    let dir = bakery();
    let o = run(
        &dir,
        &[
            "add",
            "Call the plumber due:fri",
            "--under",
            "^design",
            "--today",
            "2026-10-01",
        ],
    );
    assert!(o.status.success(), "{}", stderr(&o));
    assert_eq!(
        stdout(&o),
        "kitchen.sprig:8  - Call the plumber due:2026-10-02\n"
    );
    let kitchen = fs::read_to_string(dir.join("kitchen.sprig")).unwrap();
    assert_eq!(
        kitchen.lines().nth(7),
        Some("  - Call the plumber due:2026-10-02")
    );
}

#[test]
fn add_after_puts_the_item_after_the_whole_branch() {
    let dir = bakery();
    let o = run(&dir, &["add", "~ Pick tiles", "--after", "^design"]);
    assert!(o.status.success(), "{}", stderr(&o));
    // Design runs to line 7, its last note.
    assert_eq!(stdout(&o), "kitchen.sprig:8  ~ Pick tiles\n");
}

#[test]
fn add_with_nowhere_to_go_appends_to_plan_sprig() {
    let dir = scratch("inbox");
    fs::write(dir.join("PLAN.sprig"), "Inbox\n\n- First\n").unwrap();
    let o = run(&dir, &["add", "Second"]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert_eq!(stdout(&o), "PLAN.sprig:4  - Second\n");
    assert_eq!(
        fs::read_to_string(dir.join("PLAN.sprig")).unwrap(),
        "Inbox\n\n- First\n- Second\n"
    );
    let empty = scratch("no-inbox");
    let o = run(&empty, &["add", "Second"]);
    assert_eq!(o.status.code(), Some(2), "{}", stdout(&o));
}

// ---------------------------------------------------------------------------
// edit

#[test]
fn edit_outside_a_terminal_exits_2_without_starting_an_editor() {
    let dir = bakery();
    let marker = dir.join("editor-ran");
    let o = Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(["edit", "^loan"])
        .current_dir(&dir)
        .env("EDITOR", format!("touch {}", marker.display()))
        .env_remove("VISUAL")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    assert!(stderr(&o).contains("terminal"), "{}", stderr(&o));
    assert!(!marker.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn edit_hands_the_editor_the_line_and_the_file() {
    // `script` gives the command a terminal, as a person's shell would.
    if Command::new("script").arg("--version").output().is_err() {
        eprintln!("script is not installed here; edit was not tried");
        return;
    }
    let dir = bakery();
    let stub = dir.join("stub-editor");
    let args = dir.join("editor-args");
    fs::write(
        &stub,
        format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n", args.display()),
    )
    .unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o755)).unwrap();
    let cmd = format!("'{}' edit ^loan", env!("CARGO_BIN_EXE_sprig"));
    let o = Command::new("script")
        .args(["-qec", &cmd, "/dev/null"])
        .current_dir(&dir)
        .env("EDITOR", &stub)
        .env_remove("VISUAL")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(o.status.success(), "{}{}", stdout(&o), stderr(&o));
    let got = fs::read_to_string(&args).unwrap();
    let got: Vec<&str> = got.lines().collect();
    assert_eq!(got[0], "+13");
    assert_eq!(
        dir.join(got[1]).canonicalize().unwrap(),
        dir.join("bakery.sprig").canonicalize().unwrap()
    );
}

// ---------------------------------------------------------------------------
// import

#[test]
fn imports_match_their_fixtures_pass_check_and_survive_fmt() {
    let fixtures = repo("tests/import");
    let cases = [
        ("moving.md", "moving.sprig", &[][..]),
        ("errands.txt", "errands.sprig", &[][..]),
        (
            "sprig-prs.json",
            "sprig-prs.sprig",
            &["--title", "Sprig pull requests"][..],
        ),
    ];
    for (input, expected, extra) in cases {
        let from = fixtures.join(input);
        let o = sprig(&[&["import", from.to_str().unwrap()], extra].concat());
        assert!(o.status.success(), "{input}: {}", stderr(&o));
        let got = stdout(&o);
        let want = fs::read_to_string(fixtures.join(expected)).unwrap();
        if std::env::var_os("SPRIG_BLESS").is_some() {
            fs::write(fixtures.join(expected), &got).unwrap();
        } else {
            assert_eq!(got, want.replace("\r\n", "\n"), "{input}");
        }
        let dir = scratch("import");
        fs::write(dir.join(expected), &got).unwrap();
        let check = run(&dir, &["check"]);
        assert!(check.status.success(), "{input}: {}", stdout(&check));
        let fmt = run(&dir, &["fmt", "--check"]);
        assert!(fmt.status.success(), "{input}: {}", stdout(&fmt));
    }
}

#[test]
fn importing_from_github_needs_no_network() {
    // A user namespace with no network devices: any connection fails.
    let probe = Command::new("unshare").args(["-rn", "true"]).output();
    if !probe.is_ok_and(|o| o.status.success()) {
        eprintln!("unshare -rn is not available here; the import ran with a network");
        return;
    }
    let from = repo("tests/import/sprig-prs.json");
    let o = Command::new("unshare")
        .args([
            "-rn",
            env!("CARGO_BIN_EXE_sprig"),
            "import",
            "--from",
            "github",
        ])
        .arg(&from)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(stdout(&o).contains("x feat: v0.1, check a plan from the terminal  issue:18"));
}

#[test]
fn import_reads_the_format_from_the_extension_or_from_from() {
    let o = sprig(&["import", "tests/import/moving.md", "--from", "todotxt"]);
    assert!(o.status.success());
    assert!(stdout(&o).starts_with("moving\n"), "{}", stdout(&o));
    let dir = scratch("unknown-ext");
    fs::write(dir.join("notes.xyz"), "- [ ] a\n").unwrap();
    let o = run(&dir, &["import", "notes.xyz"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(stderr(&o).contains("--from"), "{}", stderr(&o));
}

// ---------------------------------------------------------------------------
// The GitHub Action's summary

#[cfg(unix)]
#[test]
fn the_action_summary_shows_both_done_counts_and_what_a_change_unblocked() {
    let (base, head) = (scratch("base"), scratch("head"));
    for dir in [&base, &head] {
        fs::create_dir(dir.join("plans")).unwrap();
        for e in fs::read_dir(repo("examples/bakery")).unwrap() {
            let e = e.unwrap();
            fs::copy(e.path(), dir.join("plans").join(e.file_name())).unwrap();
        }
    }
    let o = run(&head, &["mark", "^signed", "x", "--in", "plans"]);
    assert!(o.status.success(), "{}", stderr(&o));
    // A file the pull request adds has no base count.
    fs::write(head.join("plans/new.sprig"), "New\n\n- One\n").unwrap();

    let o = Command::new("bash")
        .arg(repo("action/summary.sh"))
        .args([&base, &head])
        .arg("plans")
        .env("SPRIG", env!("CARGO_BIN_EXE_sprig"))
        .env("SPRIG_TODAY", "2026-10-01")
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    let md = stdout(&o);
    assert!(
        md.contains("| `plans/lease.sprig` | 2 of 6 | **3 of 6** |"),
        "{md}"
    );
    assert!(
        md.contains("| `plans/kitchen.sprig` | 1 of 10 | 1 of 10 |"),
        "{md}"
    );
    assert!(md.contains("| `plans/new.sprig` | new | 0 of 1 |"), "{md}");
    assert!(
        md.contains("- `plans/bakery.sprig:15` Supplier credit line"),
        "{md}"
    );
    // Build still waits on Design, so it is not listed.
    assert!(!md.contains("Build"), "{md}");
}

// `find` names files relative to the folder it searched, so the summary must
// put that folder back, whether a path is a folder of folders or one file.
#[cfg(unix)]
#[test]
fn the_action_summary_names_an_unblocked_item_by_its_path_from_the_root() {
    let (base, head) = (scratch("base"), scratch("head"));
    for dir in [&base, &head] {
        fs::create_dir_all(dir.join("plans/bakery")).unwrap();
        for e in fs::read_dir(repo("examples/bakery")).unwrap() {
            let e = e.unwrap();
            fs::copy(e.path(), dir.join("plans/bakery").join(e.file_name())).unwrap();
        }
    }
    let o = run(&head, &["mark", "^signed", "x", "--in", "plans/bakery"]);
    assert!(o.status.success(), "{}", stderr(&o));

    for path in ["plans", "plans/", "plans/bakery/bakery.sprig"] {
        let o = Command::new("bash")
            .arg(repo("action/summary.sh"))
            .args([&base, &head])
            .arg(path)
            .env("SPRIG", env!("CARGO_BIN_EXE_sprig"))
            .env("SPRIG_TODAY", "2026-10-01")
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", stderr(&o));
        let md = stdout(&o);
        assert!(
            md.contains("- `plans/bakery/bakery.sprig:15` Supplier credit line"),
            "{path}: {md}"
        );
    }
}
