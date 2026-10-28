//! The README's walkthrough, run as written, so it cannot go stale.
//!
//! A ```sprig block after `<!-- file: NAME -->` is saved as NAME. An
//! `<!-- edit: NAME: OLD => NEW -->` changes that file before the next block.
//! In a ```console block, each `$ sprig ...` line is run in that directory and
//! the lines after it, up to the next `$`, are its exact standard output.

use std::fs;
use std::path::Path;
use std::process::Command;

enum Step {
    File(String, String),
    Edit(String, String, String),
    Run(Vec<String>, String),
}

fn steps(readme: &str) -> Vec<Step> {
    let mut out = Vec::new();
    let mut lines = readme.lines().peekable();
    let mut pending_file: Option<String> = None;
    while let Some(line) = lines.next() {
        if let Some(name) = line
            .strip_prefix("<!-- file: ")
            .and_then(|r| r.strip_suffix(" -->"))
        {
            pending_file = Some(name.to_string());
        } else if let Some(edit) = line
            .strip_prefix("<!-- edit: ")
            .and_then(|r| r.strip_suffix(" -->"))
        {
            let (name, change) = edit.split_once(": ").expect("edit: NAME: OLD => NEW");
            let (old, new) = change.split_once(" => ").expect("edit: NAME: OLD => NEW");
            out.push(Step::Edit(name.into(), old.into(), new.into()));
        } else if line.starts_with("```sprig") {
            let mut body = String::new();
            for l in lines.by_ref() {
                if l.starts_with("```") {
                    break;
                }
                body.push_str(l);
                body.push('\n');
            }
            if let Some(name) = pending_file.take() {
                out.push(Step::File(name, body));
            }
        } else if line.starts_with("```console") {
            let mut block: Vec<&str> = Vec::new();
            for l in lines.by_ref() {
                if l.starts_with("```") {
                    break;
                }
                block.push(l);
            }
            let mut i = 0;
            while i < block.len() {
                let command = block[i]
                    .strip_prefix("$ ")
                    .expect("a console block starts with $");
                let args: Vec<String> = command.split_whitespace().map(String::from).collect();
                assert_eq!(args[0], "sprig", "the README runs only sprig: {command}");
                i += 1;
                let mut expected = String::new();
                while i < block.len() && !block[i].starts_with("$ ") {
                    expected.push_str(block[i]);
                    expected.push('\n');
                    i += 1;
                }
                out.push(Step::Run(args[1..].to_vec(), expected));
            }
        }
    }
    out
}

#[test]
fn every_command_in_the_readme_prints_what_the_readme_says() {
    let readme =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../README.md")).unwrap();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("readme-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let steps = steps(&readme);
    let runs = steps.iter().filter(|s| matches!(s, Step::Run(..))).count();
    assert!(runs >= 3, "the README walkthrough was not found");
    for step in steps {
        match step {
            Step::File(name, body) => fs::write(dir.join(name), body).unwrap(),
            Step::Edit(name, old, new) => {
                let path = dir.join(&name);
                let text = fs::read_to_string(&path).unwrap();
                assert!(text.contains(&old), "{name} does not contain {old}");
                fs::write(path, text.replacen(&old, &new, 1)).unwrap();
            }
            Step::Run(args, expected) => {
                // The plan has no dates, so the day matters only to keep the
                // run reproducible.
                let out = Command::new(env!("CARGO_BIN_EXE_sprig"))
                    .args(&args)
                    .current_dir(&dir)
                    .env("SPRIG_TODAY", "2026-10-01")
                    .output()
                    .unwrap();
                assert_ne!(
                    out.status.code(),
                    Some(2),
                    "sprig {}: {}",
                    args.join(" "),
                    String::from_utf8_lossy(&out.stderr)
                );
                assert_eq!(
                    String::from_utf8_lossy(&out.stdout),
                    expected,
                    "sprig {} printed something other than the README says",
                    args.join(" ")
                );
            }
        }
    }
}
