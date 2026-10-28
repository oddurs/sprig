//! The walkthroughs in the README and in the site's first-plan tutorial, run
//! as written, so they cannot go stale.
//!
//! A ```sprig block after `<!-- file: NAME -->` (in MDX, `{/* file: NAME */}`)
//! is saved as NAME. An `<!-- edit: NAME: OLD => NEW -->` changes that file
//! before the next block.
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

/// The text of a `<!-- kind: ... -->` or `{/* kind: ... */}` line.
fn marker<'a>(line: &'a str, kind: &str) -> Option<&'a str> {
    let html = line
        .strip_prefix("<!-- ")
        .and_then(|r| r.strip_suffix(" -->"));
    let mdx = line
        .strip_prefix("{/* ")
        .and_then(|r| r.strip_suffix(" */}"));
    html.or(mdx)?.strip_prefix(kind)?.strip_prefix(": ")
}

fn steps(readme: &str) -> Vec<Step> {
    let mut out = Vec::new();
    let mut lines = readme.lines().peekable();
    let mut pending_file: Option<String> = None;
    while let Some(line) = lines.next() {
        if let Some(name) = marker(line, "file") {
            pending_file = Some(name.to_string());
        } else if let Some(edit) = marker(line, "edit") {
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

fn run_walkthrough(path: &str) {
    let text = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(path),
    )
    .unwrap();
    let name = path.replace(['/', '.'], "-");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let steps = steps(&text);
    let runs = steps.iter().filter(|s| matches!(s, Step::Run(..))).count();
    assert!(runs >= 3, "{path}: the walkthrough was not found");
    for step in steps {
        match step {
            Step::File(name, body) => fs::write(dir.join(name), body).unwrap(),
            Step::Edit(name, old, new) => {
                let file = dir.join(&name);
                let text = fs::read_to_string(&file).unwrap();
                assert!(text.contains(&old), "{path}: {name} does not contain {old}");
                fs::write(file, text.replacen(&old, &new, 1)).unwrap();
            }
            Step::Run(args, expected) => {
                // The plans have no dates, so the day matters only to keep
                // the run reproducible.
                let out = Command::new(env!("CARGO_BIN_EXE_sprig"))
                    .args(&args)
                    .current_dir(&dir)
                    .env("SPRIG_TODAY", "2026-10-01")
                    .output()
                    .unwrap();
                assert_ne!(
                    out.status.code(),
                    Some(2),
                    "{path}: sprig {}: {}",
                    args.join(" "),
                    String::from_utf8_lossy(&out.stderr)
                );
                assert_eq!(
                    String::from_utf8_lossy(&out.stdout),
                    expected,
                    "{path}: sprig {} printed something other than the page says",
                    args.join(" ")
                );
            }
        }
    }
}

#[test]
fn every_command_in_the_readme_prints_what_the_readme_says() {
    run_walkthrough("README.md");
}

#[test]
fn every_command_in_the_first_plan_tutorial_prints_what_it_says() {
    run_walkthrough("site/src/content/docs/docs/first-plan.mdx");
}
