//! `spec/diagnostics.md` and the core agree on every code: what the core can
//! emit is listed, and what is listed exists, with the same severity and rule.

use std::fs;
use std::path::Path;

use sprig_core::Code;

fn catalogue() -> Vec<(String, String, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/diagnostics.md");
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .filter(|l| l.starts_with("| S"))
        .map(|l| {
            let cells: Vec<&str> = l.trim_matches('|').split('|').map(str::trim).collect();
            (
                cells[0].to_string(),
                cells[1].to_string(),
                cells[2].to_string(),
            )
        })
        .collect()
}

#[test]
fn every_code_the_core_emits_is_in_the_catalogue() {
    let listed = catalogue();
    for code in Code::ALL {
        let row = listed
            .iter()
            .find(|(c, ..)| c == code.as_str())
            .unwrap_or_else(|| panic!("{code} is not in spec/diagnostics.md"));
        assert_eq!(row.1, code.severity().as_str(), "{code}: severity");
        assert_eq!(row.2, code.rule(), "{code}: rule");
    }
}

#[test]
fn every_code_in_the_catalogue_exists_in_the_core() {
    for (code, ..) in catalogue() {
        assert!(
            Code::ALL.iter().any(|c| c.as_str() == code),
            "{code} is catalogued but the core has no such code; a retired code keeps its row marked retired"
        );
    }
}

#[test]
fn the_catalogue_lists_each_code_once_in_order() {
    let codes: Vec<String> = catalogue().into_iter().map(|(c, ..)| c).collect();
    let mut sorted = codes.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(codes, sorted);
}

/// Every code the catalogue has ever published. A new code is added here when
/// it is catalogued; none is ever taken out, so a code cannot be dropped and
/// later reused for something else (docs/stability.md).
const PUBLISHED: &[&str] = &[
    "S0001", "S0002", "S0003", "S0004", "S0005", "S0006", "S0007", "S0101", "S0102", "S0103",
    "S0104", "S0105", "S0201", "S0202", "S0203", "S0204", "S0205", "S0206",
];

#[test]
fn a_published_code_stays_in_the_catalogue_with_its_meaning() {
    let codes: Vec<String> = catalogue().into_iter().map(|(c, ..)| c).collect();
    for code in PUBLISHED {
        assert!(
            codes.iter().any(|c| c == code),
            "{code} was published and must stay catalogued"
        );
    }
    for code in &codes {
        assert!(
            PUBLISHED.contains(&code.as_str()),
            "{code} is new: add it to PUBLISHED, where it stays"
        );
    }
}

#[test]
fn every_code_has_a_note_that_explains_it() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/diagnostics.md");
    let text = fs::read_to_string(path).unwrap();
    let notes = text
        .split("## Notes on each code")
        .nth(1)
        .expect("a notes section");
    for code in Code::ALL {
        // A note may cover several codes that share a cause: **S0101, S0102.**
        let note = notes
            .split("\n\n")
            .filter_map(|p| p.strip_prefix("**").and_then(|r| r.split_once(".**")))
            .find(|(head, _)| head.split(", ").any(|c| c == code.as_str()))
            .unwrap_or_else(|| panic!("spec/diagnostics.md has no note for {code}"));
        assert!(note.1.len() > 40, "the note for {code} says too little");
    }
}
