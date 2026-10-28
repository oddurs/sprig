//! The specification says its examples are valid Sprig files. Each fenced
//! `sprig` block in `spec/sprig.md` is read here, so an example that stops
//! meaning what the text around it says is caught.

use std::fs;
use std::path::Path;

use sprig_core::{Severity, Workspace, parse};

fn examples() -> Vec<(usize, String)> {
    let spec =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/sprig.md"))
            .unwrap();
    let mut out = Vec::new();
    let mut block: Option<(usize, String)> = None;
    for (i, line) in spec.lines().enumerate() {
        match &mut block {
            None if line == "```sprig" => block = Some((i + 2, String::new())),
            Some((at, text)) if line == "```" => {
                out.push((*at, std::mem::take(text)));
                block = None;
            }
            Some((_, text)) => {
                text.push_str(line);
                text.push('\n');
            }
            None => {}
        }
    }
    out
}

#[test]
fn every_example_in_the_spec_parses_without_an_error() {
    let examples = examples();
    assert!(examples.len() > 50, "the spec's examples were not found");
    for (line, text) in examples {
        let errors: Vec<String> = parse("example.sprig", &text)
            .diagnostics
            .iter()
            .filter(|d| d.severity() == Severity::Error)
            .map(ToString::to_string)
            .collect();
        assert!(errors.is_empty(), "spec/sprig.md:{line}: {errors:?}");
    }
}

#[test]
fn the_dropped_reference_example_does_not_wait() {
    let text = examples()
        .into_iter()
        .map(|(_, t)| t)
        .find(|t| t.starts_with("Dropped\n"))
        .expect("§5.7 has a Dropped example");
    let mut w = Workspace::new();
    w.insert("example.sprig", text);
    let today = sprig_core::Date::parse_iso("2026-10-01").unwrap();
    let r = w.resolve(today);
    let launch = &r.tree("example.sprig").unwrap().root.children[1];
    assert_eq!(launch.label, "Launch");
    assert!(!launch.blocked, "a dropped item holds nothing up (§5.7)");
}
