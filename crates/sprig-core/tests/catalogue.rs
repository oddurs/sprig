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
