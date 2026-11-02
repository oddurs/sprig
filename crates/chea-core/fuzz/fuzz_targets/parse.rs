//! Any bytes, as a file: parsing, resolving and every view of the result
//! must return, never panic (spec: user input never panics).
#![no_main]

use libfuzzer_sys::fuzz_target;
use chea_core::{Date, Workspace, json, syntax};

fuzz_target!(|data: &[u8]| {
    let Ok(src) = std::str::from_utf8(data) else {
        return;
    };
    let today = Date::new(2026, 10, 1).unwrap();
    // A file that grafts itself and one other, so resolution has work to do.
    let mut ws = Workspace::new();
    ws.insert("main.chea", src);
    ws.insert("plans/other.chea", src);
    let res = ws.resolve(today);
    for file in ["main.chea", "plans/other.chea"] {
        let _ = json::tree_document(&res, file, today);
    }
    for line in syntax::lex(src) {
        assert!(line.span.end <= src.len());
    }
    for line in src.lines() {
        let _ = syntax::tick(line, today);
        let _ = syntax::expand_dates(line, today);
    }
});
