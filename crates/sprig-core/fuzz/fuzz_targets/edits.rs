//! Random edits on a file: each either fails cleanly or changes only the
//! bytes it reports changing, and what it writes reads back.
#![no_main]

use libfuzzer_sys::fuzz_target;
use sprig_core::edit::{self, Change, Place};
use sprig_core::{Date, Mark, parse};

const MARKS: [Mark; 6] = [
    Mark::Todo,
    Mark::Doing,
    Mark::Done,
    Mark::Dropped,
    Mark::Later,
    Mark::Question,
];

/// Every byte outside the change's spans is where it was.
fn preserves(before: &str, c: &Change) {
    assert_eq!(
        edit::apply(before, &c.edits),
        c.source,
        "the edits do not make the source"
    );
    let mut at = 0;
    let mut out = 0;
    for e in &c.edits {
        let kept = &before[at..e.span.start];
        assert_eq!(
            &c.source[out..out + kept.len()],
            kept,
            "bytes outside an edit moved"
        );
        out += kept.len() + e.text.len();
        at = e.span.end;
    }
    assert_eq!(&c.source[out..], &before[at..]);
}

fuzz_target!(|data: &[u8]| {
    // The first half is the file, the rest the edits to make, a byte each.
    let split = data.len() / 2;
    let Ok(src) = std::str::from_utf8(&data[..split]) else {
        return;
    };
    let today = Date::new(2026, 10, 1).unwrap();
    let mut text = src.to_string();
    for op in data[split..].chunks(2) {
        let lines = text.lines().count().max(1);
        let line = 1 + usize::from(op.get(1).copied().unwrap_or(0)) % lines;
        let result = match op[0] % 7 {
            0 => edit::tick(&text, line, today).map(|(c, _)| c),
            1 => edit::set_mark(&text, line, MARKS[usize::from(op[0] / 7) % MARKS.len()]),
            2 => edit::set_field(&text, line, "due", Some("2026-10-02")),
            3 => edit::set_field(&text, line, "note", Some("two words")),
            4 => edit::add_item(&text, line, Place::Child, "- added  @fuzz"),
            5 => edit::add_answer(&text, line, "settled"),
            _ => Ok(edit::format(&text, today)),
        };
        if let Ok(c) = result {
            preserves(&text, &c);
            // What an edit writes is a file the parser reads.
            let _ = parse("main.sprig", &c.source);
            text = c.source;
        }
    }
    // Formatting a formatted file changes nothing (spec/tools.md T2).
    let once = edit::format(&text, today).source;
    assert_eq!(
        edit::format(&once, today).source,
        once,
        "format is not idempotent"
    );
});
