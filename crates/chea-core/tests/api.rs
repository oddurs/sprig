//! Every public function of the core is shown working in a doc example,
//! which `cargo test` runs, or is named below with the reason it is not.

use std::fs;
use std::path::Path;

/// Public functions without an example, each with the reason. None, today:
/// a new function joins an example, or this list with a reason.
const EXEMPT: &[(&str, &str)] = &[];

fn sources() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out: Vec<(String, String)> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .map(|p| {
            (
                p.file_name().unwrap().to_string_lossy().into_owned(),
                fs::read_to_string(&p).unwrap(),
            )
        })
        .collect();
    out.sort();
    out
}

/// The code inside the doc comments' fenced blocks.
fn examples(src: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in src.lines() {
        let t = line.trim_start();
        let Some(doc) = t.strip_prefix("//!").or_else(|| t.strip_prefix("///")) else {
            inside = false;
            continue;
        };
        let doc = doc.strip_prefix(' ').unwrap_or(doc);
        if doc.starts_with("```") {
            inside = !inside;
        } else if inside {
            out.push_str(doc);
            out.push('\n');
        }
    }
    out
}

/// The names of the public functions, outside test modules.
fn public_fns(src: &str) -> Vec<String> {
    let body = src.split("#[cfg(test)]").next().unwrap();
    body.lines()
        .filter_map(|l| l.trim_start().strip_prefix("pub fn "))
        .map(|rest| rest.split(['(', '<']).next().unwrap().to_string())
        .collect()
}

#[test]
fn every_public_function_has_an_example_or_a_reason() {
    let all: Vec<(String, String)> = sources();
    let code: String = all.iter().map(|(_, s)| examples(s)).collect();
    let mut missing = Vec::new();
    for (file, src) in &all {
        for name in public_fns(src) {
            let shown = code.contains(&format!("{name}(")) || code.contains(&format!("::{name}"));
            if !shown && !EXEMPT.iter().any(|(n, _)| *n == name) {
                missing.push(format!("{file}: {name}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "no example calls:\n  {}",
        missing.join("\n  ")
    );
    // The scan found the API, not nothing.
    let count: usize = all.iter().map(|(_, s)| public_fns(s).len()).sum();
    assert!(count > 70, "found only {count} public functions");
}
