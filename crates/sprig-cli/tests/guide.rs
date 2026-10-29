//! The recipes in the site's shell guide, run as written in a copy of the
//! bakery, so the guide cannot go stale.
//!
//! A ```sh block after `<!-- run -->` runs in bash and must succeed; after
//! `<!-- run: needs jq dot -->` it runs only where those commands exist,
//! unless `SPRIG_CI_TOOLS` says they were installed, when a missing one fails.
//! Blocks run in order in one folder, so each sees what the last one wrote.

#![cfg(unix)]

use std::fs;
use std::path::Path;
use std::process::Command;

fn blocks(guide: &str) -> Vec<(Vec<String>, String)> {
    let mut out = Vec::new();
    let mut lines = guide.lines();
    while let Some(line) = lines.next() {
        let Some(rest) = line
            .strip_prefix("<!-- run")
            .and_then(|r| r.strip_suffix(" -->"))
        else {
            continue;
        };
        let needs = rest
            .strip_prefix(": needs ")
            .map(|n| n.split_whitespace().map(String::from).collect())
            .unwrap_or_default();
        assert_eq!(
            lines.next(),
            Some("```sh"),
            "a run marker is followed by a sh block"
        );
        let body: Vec<&str> = lines.by_ref().take_while(|l| *l != "```").collect();
        out.push((needs, body.join("\n") + "\n"));
    }
    out
}

fn have(cmd: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {cmd}")])
        .output()
        .is_ok_and(|o| o.status.success())
}

#[test]
fn every_recipe_in_the_shell_guide_runs_against_the_bakery() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let guide = fs::read_to_string(root.join("site/src/content/docs/docs/shell.md")).unwrap();
    let blocks = blocks(&guide);
    assert!(blocks.len() >= 8, "found {} runnable recipes", blocks.len());

    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("guide-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    for e in fs::read_dir(root.join("examples/bakery")).unwrap() {
        let e = e.unwrap();
        fs::copy(e.path(), dir.join(e.file_name())).unwrap();
    }
    // The pre-commit recipe reads what is staged.
    let git = Command::new("sh")
        .args(["-c", "git init -q && git add -A"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        git.status.success(),
        "{}",
        String::from_utf8_lossy(&git.stderr)
    );

    // fzf asks a person; this one picks the first line, as Enter would.
    let stubs = dir.join(".stubs");
    fs::create_dir(&stubs).unwrap();
    fs::write(stubs.join("fzf"), "#!/bin/sh\nhead -n 1\n").unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(stubs.join("fzf"), fs::Permissions::from_mode(0o755)).unwrap();
    let bin = Path::new(env!("CARGO_BIN_EXE_sprig")).parent().unwrap();
    let path = format!(
        "{}:{}:{}",
        bin.display(),
        stubs.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let required = std::env::var_os("SPRIG_CI_TOOLS").is_some_and(|v| !v.is_empty());

    for (needs, body) in blocks {
        let missing: Vec<&String> = needs.iter().filter(|c| !have(c)).collect();
        if !missing.is_empty() {
            assert!(
                !required,
                "SPRIG_CI_TOOLS is set but {missing:?} is not installed"
            );
            eprintln!("skipped, without {missing:?}:\n{body}");
            continue;
        }
        let o = Command::new("bash")
            .args(["-euo", "pipefail", "-c", &body])
            .current_dir(&dir)
            .env("PATH", &path)
            .env("SPRIG_TODAY", "2026-10-01")
            .env("NO_COLOR", "1")
            .output()
            .unwrap();
        assert!(
            o.status.success(),
            "{body}\nexited {:?}:\n{}{}",
            o.status.code(),
            String::from_utf8_lossy(&o.stdout),
            String::from_utf8_lossy(&o.stderr)
        );
    }
    // The recipes did what they say.
    let kitchen = fs::read_to_string(dir.join("kitchen.sprig")).unwrap();
    assert!(kitchen.contains("~ Design  ^design"), "{kitchen}");
    let launch = fs::read_to_string(dir.join("launch.sprig")).unwrap();
    assert!(launch.contains("x Logo"), "{launch}");
    assert!(
        fs::read_to_string(dir.join("plans.ics"))
            .unwrap()
            .starts_with("BEGIN:VCALENDAR")
    );
    if have("dot") {
        assert!(
            fs::read_to_string(dir.join("plan.svg"))
                .unwrap()
                .contains("<svg")
        );
    }
}
