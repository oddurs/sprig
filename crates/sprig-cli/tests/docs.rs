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
