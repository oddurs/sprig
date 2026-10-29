//! Simplicity, held like any other promise: the size of the format, the
//! command and the spec, counted from the things themselves (cairn 0155).
//!
//! Each count is pinned. Adding a mark, a command, a flag, a field with a
//! meaning, a diagnostic code or an MCP tool fails here until its number is
//! raised in this file, in the same pull request, where a reviewer sees it.
//! Taking one away fails too, until the number comes down: the budget only
//! ever records what is true. The spec and the card have ceilings rather than
//! exact sizes, since rewording a sentence is not growth.

use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// What Sprig is allowed to be.
const MARKS: usize = 9;
const FIELDS_WITH_A_MEANING: usize = 5;
const COMMANDS: usize = 13;
const FLAGS: usize = 26;
const DIAGNOSTIC_CODES: usize = 18;
const MCP_TOOLS: usize = 10;
const CARD_LINES_AT_MOST: usize = 32;
const CORE_SPEC_WORDS_AT_MOST: usize = 3400;

fn repo(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn read(path: &str) -> String {
    fs::read_to_string(repo(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn sprig(args: &[&str]) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(args)
        .output()
        .unwrap();
    String::from_utf8(o.stdout).unwrap()
}

fn pinned(what: &str, counted: usize, budget: usize) {
    assert!(
        counted == budget,
        "{what}: {counted}, and the budget says {budget}. {}",
        if counted > budget {
            "Raise it in crates/sprig-cli/tests/budget.rs, and say in the pull request why it is worth it."
        } else {
            "Lower it in crates/sprig-cli/tests/budget.rs, so the budget stays true."
        }
    );
}

/// The card: the whole format, between its markers in `spec/sprig.md`.
fn card() -> String {
    let spec = read("spec/sprig.md");
    let start = "<!-- card:start -->";
    let a = spec.find(start).expect("the spec has the card") + start.len();
    let b = spec.find("<!-- card:end -->").expect("the card ends");
    spec[a..b].trim_matches('\n').to_string()
}

/// The backticked words of a piece of text.
fn quoted(text: &str) -> Vec<&str> {
    text.split('`').skip(1).step_by(2).collect()
}

/// The section of the spec under one heading, up to the next of any level.
fn section(heading: &str) -> String {
    let spec = read("spec/sprig.md");
    let a = spec
        .find(heading)
        .unwrap_or_else(|| panic!("the spec has {heading}"));
    let rest = &spec[a + heading.len()..];
    let b = rest.find("\n#").unwrap_or(rest.len());
    rest[..b].to_string()
}

// ---------------------------------------------------------------------------
// The format

#[test]
fn the_format_has_nine_marks_and_the_card_shows_each() {
    // Marks as §2.2 defines them, from the first column of its table.
    let defined: BTreeSet<String> = section("### 2.2 Marks")
        .lines()
        .filter(|l| l.starts_with("| `"))
        .filter_map(|l| quoted(l).first().map(|m| m.to_string()))
        .collect();
    pinned("marks", defined.len(), MARKS);
    // The card's first table is the lines; the second is the tokens.
    let shown: BTreeSet<String> = card()
        .lines()
        .take_while(|l| !l.is_empty())
        .filter(|l| l.starts_with("| `"))
        .filter_map(|l| quoted(l).first().map(|m| m.trim().to_string()))
        .filter(|m| m.chars().count() == 1)
        .collect();
    assert_eq!(shown, defined, "the card's marks and §2.2's");
}

#[test]
fn the_fields_with_a_meaning_are_the_ones_the_card_names() {
    // §4.6 names the keys with a meaning; §4.7 adds after.
    let sentence = section("### 4.6 Fields")
        .lines()
        .skip_while(|l| !l.starts_with("The keys"))
        .take_while(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut defined: BTreeSet<String> = quoted(&sentence).iter().map(|k| k.to_string()).collect();
    assert!(defined.len() > 2, "§4.6's list of keys was not found");
    defined.insert("after".into());
    pinned(
        "fields with a meaning",
        defined.len(),
        FIELDS_WITH_A_MEANING,
    );
    let card = card();
    for k in &defined {
        assert!(
            card.contains(&format!("`{k}:")),
            "the card does not show {k}:"
        );
    }
}

#[test]
fn the_card_fits_on_one_screen_and_the_readme_shows_it_as_it_is() {
    let card = card();
    let lines = card.lines().count();
    assert!(
        lines <= CARD_LINES_AT_MOST,
        "the card is {lines} lines; the budget is {CARD_LINES_AT_MOST}"
    );
    assert!(
        read("README.md").contains(&card),
        "copy the card from spec/sprig.md into README.md, markers and all"
    );
}

#[test]
fn the_core_of_the_spec_reads_in_one_sitting() {
    // Prose only: the examples are files, read at a glance.
    let spec = read("spec/sprig.md");
    let mut words = 0;
    let mut fenced = false;
    for line in spec.lines() {
        if line.starts_with("```") {
            fenced = !fenced;
        } else if !fenced {
            words += line.split_whitespace().count();
        }
    }
    assert!(
        words <= CORE_SPEC_WORDS_AT_MOST,
        "spec/sprig.md is {words} words of prose; the budget is {CORE_SPEC_WORDS_AT_MOST}. \
         A ruling on an edge case belongs in spec/annex.md, under its rule's number."
    );
}

#[test]
fn every_diagnostic_code_is_counted() {
    let codes = read("spec/diagnostics.md")
        .lines()
        .filter(|l| l.starts_with("| S") && l[3..].starts_with(|c: char| c.is_ascii_digit()))
        .count();
    pinned("diagnostic codes", codes, DIAGNOSTIC_CODES);
}

// ---------------------------------------------------------------------------
// The command

/// The commands `sprig --help` lists.
fn commands() -> Vec<String> {
    sprig(&["--help"])
        .lines()
        .skip_while(|l| !l.starts_with("Commands:"))
        .skip(1)
        .take_while(|l| !l.trim().is_empty())
        .filter_map(|l| l.split_whitespace().next().map(String::from))
        .collect()
}

#[test]
fn the_command_answers_thirteen_questions() {
    pinned("commands", commands().len(), COMMANDS);
}

#[test]
fn the_commands_have_twenty_six_flags_between_them() {
    // Each command's own options; the global ones are counted once, here.
    let global = ["--today", "--color", "--plain", "--help", "--version"];
    let flags: usize = commands()
        .iter()
        .map(|c| {
            sprig(&[c, "--help"])
                .lines()
                .skip_while(|l| !l.starts_with("Options:"))
                .filter(|l| {
                    // An option's own line, not help text that mentions one.
                    let l = l.trim_start();
                    if !l.starts_with('-') {
                        return false;
                    }
                    let name = l.split(&[' ', ','][..]).find(|w| w.starts_with("--"));
                    name.is_some_and(|n| !global.contains(&n))
                })
                .count()
        })
        .sum();
    pinned("flags", flags, FLAGS);
}

#[test]
fn the_mcp_server_offers_ten_tools() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sprig"))
        .args(["mcp", "--in"])
        .arg(repo("examples/bakery"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            concat!(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"budget","version":"0"}}}"#,
                "\n",
                r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
                "\n",
                r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
                "\n",
            )
            .as_bytes(),
        )
        .unwrap();
    let out = String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap();
    let list: serde_json::Value = out
        .lines()
        .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
        .find(|v| v["id"] == 2)
        .expect("tools/list answers");
    let tools = list["result"]["tools"]
        .as_array()
        .expect("a list of tools")
        .len();
    pinned("MCP tools", tools, MCP_TOOLS);
}
