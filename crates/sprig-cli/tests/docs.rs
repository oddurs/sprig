//! The documents that make promises name the tests and examples that keep
//! them: here, each name is held to exist, so a promise cannot outlive its
//! test unnoticed.

use std::fs;
use std::path::{Path, PathBuf};

fn repo(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn read(path: &str) -> String {
    fs::read_to_string(repo(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// The backticked words of a document.
fn quoted(doc: &str) -> Vec<String> {
    doc.split('`')
        .skip(1)
        .step_by(2)
        .map(String::from)
        .collect()
}

#[test]
fn every_test_the_stability_policy_names_exists() {
    let doc = read("docs/stability.md");
    let words = quoted(&doc);
    let mut named = 0;
    for pair in words.windows(2) {
        let (file, test) = (&pair[0], &pair[1]);
        if !(file.ends_with(".rs") && file.contains("tests/")) {
            continue;
        }
        if !test
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            continue;
        }
        let src = read(file);
        assert!(
            src.contains(&format!("fn {test}(")),
            "{file} has no test {test}"
        );
        named += 1;
    }
    assert!(named >= 15, "found only {named} named tests");
}

#[test]
fn every_example_the_stability_policy_and_the_rfcs_cite_exists() {
    let mut docs = vec![read("docs/stability.md")];
    for e in fs::read_dir(repo("rfcs")).unwrap() {
        docs.push(fs::read_to_string(e.unwrap().path()).unwrap());
    }
    for doc in docs {
        for w in quoted(&doc) {
            if ["lines/", "tokens/", "graph/"]
                .iter()
                .any(|s| w.starts_with(s))
            {
                assert!(
                    repo("tests/conformance").join(&w).is_dir(),
                    "no example {w}"
                );
            }
        }
    }
}

#[test]
fn the_rfc_index_lists_every_rfc() {
    let index = read("rfcs/README.md");
    for e in fs::read_dir(repo("rfcs")).unwrap() {
        let name = e.unwrap().file_name().to_string_lossy().into_owned();
        if name == "README.md" || name.starts_with("0000") {
            continue;
        }
        assert!(
            index.contains(&format!("]({name})")),
            "rfcs/README.md does not list {name}"
        );
    }
}

#[test]
fn the_policy_is_linked_from_the_readme_the_spec_and_the_crate() {
    for (file, link) in [
        ("README.md", "docs/stability.md"),
        ("spec/sprig.md", "docs/stability.md"),
        ("crates/sprig-core/src/lib.rs", "docs/stability.md"),
        ("CONTRIBUTING.md", "rfcs/"),
    ] {
        assert!(read(file).contains(link), "{file} does not link {link}");
    }
}

/// Every file git tracks, and every folder that holds one.
fn tracked() -> (Vec<String>, std::collections::BTreeSet<String>) {
    let o = std::process::Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(repo(""))
        .output()
        .expect("git runs");
    assert!(o.status.success());
    let files: Vec<String> = String::from_utf8(o.stdout)
        .unwrap()
        .split('\0')
        .filter(|f| !f.is_empty())
        .map(String::from)
        .collect();
    let mut known: std::collections::BTreeSet<String> = files.iter().cloned().collect();
    for f in &files {
        let mut p = f.as_str();
        while let Some((dir, _)) = p.rsplit_once('/') {
            known.insert(dir.to_string());
            p = dir;
        }
    }
    (files, known)
}

/// `a/./b/../c` as `a/c`, the way a link is followed.
fn normal(path: &str) -> String {
    let mut out: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                out.pop();
            }
            p => out.push(p),
        }
    }
    out.join("/")
}

/// The targets of a document's relative Markdown links, `[text](target)`.
fn links(line: &str) -> Vec<&str> {
    line.match_indices("](")
        .filter_map(|(at, _)| {
            let rest = &line[at + 2..];
            let end = rest.find(')')?;
            let t = rest[..end].split('#').next().unwrap();
            let external = t.contains(':') || t.starts_with('/') || t.contains(char::is_whitespace);
            (!t.is_empty() && !external).then_some(t)
        })
        .collect()
}

/// Backticked words that name something in the repository, like
/// `spec/sprig.md`: a top-level folder, a slash, and no placeholders.
fn paths(line: &str) -> Vec<&str> {
    const TOP: &[&str] = &[
        "action/",
        "cairn/",
        "crates/",
        "design/",
        "docs/",
        "editors/",
        "examples/",
        "rfcs/",
        "scripts/",
        "site/",
        "spec/",
        "tests/",
        ".github/",
        ".githooks/",
    ];
    line.split('`')
        .skip(1)
        .step_by(2)
        .map(|w| w.trim_end_matches(['.', ',', ':', ';']))
        .filter(|w| TOP.iter().any(|t| w.starts_with(t)))
        .filter(|w| !w.contains([' ', '<', '>', '*', '{', '$']))
        .collect()
}

// A guide that points at a file someone renamed sends the reader nowhere, and
// nothing else notices. A path may be from the repository's root or from the
// package the document is in (`tests/snapshots/` in a crate), and may name
// something gitignored that a build writes. The cairn items are left out:
// they record what was true when they were written.
#[test]
fn every_path_a_document_names_exists() {
    let (files, known) = tracked();
    let ignored = |p: &str| {
        std::process::Command::new("git")
            .args(["check-ignore", "-q", "--no-index", p])
            .current_dir(repo(""))
            .status()
            .unwrap()
            .success()
    };
    let packages: Vec<&str> = files
        .iter()
        .filter_map(|f| {
            f.strip_suffix("Cargo.toml")
                .or_else(|| f.strip_suffix("package.json"))
        })
        .map(|d| d.trim_end_matches('/'))
        .collect();
    let mut missing = Vec::new();
    for f in &files {
        let prose = f.ends_with(".md") || f.ends_with(".mdx");
        let skip = f.starts_with("cairn/items/")
            || (f.starts_with("tests/conformance/") && !f.ends_with("README.md"));
        if skip || !(prose || f.ends_with(".rs")) {
            continue;
        }
        let dir = f.rsplit_once('/').map_or("", |(d, _)| d);
        let package = packages
            .iter()
            .filter(|p| p.is_empty() || dir == **p || dir.starts_with(&format!("{p}/")))
            .max_by_key(|p| p.len())
            .copied()
            .unwrap_or("");
        let text = read(f);
        for (n, line) in text.lines().enumerate() {
            let mut found: Vec<(String, Vec<String>)> = Vec::new();
            if prose {
                for t in links(line) {
                    found.push((t.to_string(), vec![normal(&format!("{dir}/{t}"))]));
                }
            }
            for t in paths(line) {
                found.push((
                    t.to_string(),
                    vec![normal(t), normal(&format!("{package}/{t}"))],
                ));
            }
            for (t, candidates) in found {
                if !candidates.iter().any(|c| known.contains(c) || ignored(c)) {
                    missing.push(format!("{f}:{}: {t}", n + 1));
                }
            }
        }
    }
    assert!(
        missing.is_empty(),
        "these name nothing:\n{}",
        missing.join("\n")
    );
}

#[test]
fn a_renamed_file_is_reported_and_a_link_to_a_folder_is_not() {
    assert_eq!(
        links("see [the spec](spec/sprig.md#2) and [x](https://a.b/c)"),
        ["spec/sprig.md"]
    );
    assert_eq!(
        links("[a](../README.md) [b](#top) [c](/docs/)"),
        ["../README.md"]
    );
    assert_eq!(
        paths("in `spec/sprig.md`, and `site/<name>/` or `scripts/task check`"),
        ["spec/sprig.md"]
    );
    assert_eq!(normal("crates/x/../../spec/./sprig.md"), "spec/sprig.md");
}

// A lockfile Dependabot does not watch never hears of an advisory, and adding
// one is easy to do without noticing: the editors and the MCP test each
// brought their own.
#[test]
fn dependabot_watches_every_lockfile() {
    let config = read(".github/dependabot.yml");
    let (files, _) = tracked();
    for f in &files {
        let dir = match f.as_str() {
            "Cargo.lock" | "package-lock.json" => "/".to_string(),
            f => match f
                .strip_suffix("/Cargo.lock")
                .or_else(|| f.strip_suffix("/package-lock.json"))
            {
                Some(d) => format!("/{d}"),
                None => continue,
            },
        };
        assert!(
            config.contains(&format!("\"{dir}\"")),
            "{f} is not in .github/dependabot.yml"
        );
    }
}
