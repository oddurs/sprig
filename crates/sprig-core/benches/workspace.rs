//! Times parse, resolve and check on a generated 5,000-line workspace, and
//! compares the medians with the committed baseline (`baseline.json`):
//! more than twice the baseline fails. See README.md in this directory.
//!
//! ```sh
//! cargo bench -p sprig-core --bench workspace            # compare
//! cargo bench -p sprig-core --bench workspace -- --save  # write the baseline
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::hint::black_box;
use std::path::Path;
use std::time::Instant;

use sprig_core::{Date, Workspace, syntax};

const FILES: usize = 20;
const LINES: usize = 250;
const RUNS: usize = 31;
const SLOWER: f64 = 2.0;

/// One plan grafting the other nineteen, as `examples/bakery/` does, with
/// people, dates, estimates, groups, notes and a reference into the next
/// file on every item, so every part of resolution has work to do.
fn workspace() -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    for i in 0..FILES {
        let mut s = format!("Part {i}  @p{i}  due:2026-12-01\n\n");
        if i == 0 {
            for j in 1..FILES {
                s += &format!("+ [[part{j}]]\n");
            }
        }
        let mut n = 0;
        while s.lines().count() < LINES {
            n += 1;
            s += &format!("# Area {n}\n");
            s += &format!(
                "  - Task {n}  ^t{n}  est:2h  @p{}  after:[[part{}^t{n}]]\n",
                n % 7,
                (i + 1) % FILES
            );
            s += "    Notes about the task, long enough to be a paragraph.\n";
            s += &format!("    x Step one  due:2026-11-0{}\n", n % 9 + 1);
            s += "    ~ Step two  start:2026-10-15\n";
            s += &format!("  ? Question {n}  ^q{n}\n    = An answer.\n");
        }
        files.insert(format!("part{i}.sprig"), s);
    }
    files
}

fn median(mut f: impl FnMut()) -> f64 {
    let mut times: Vec<f64> = (0..RUNS)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    times.sort_by(f64::total_cmp);
    times[RUNS / 2]
}

fn main() {
    let files = workspace();
    let lines: usize = files.values().map(|s| s.lines().count()).sum();
    let today = Date::parse_iso("2026-10-01").expect("a date");
    let mut ws = Workspace::new();
    for (k, v) in &files {
        ws.insert(k.clone(), v.clone());
    }

    // `cargo test --all-targets` runs this too, in a debug build: then it
    // checks that every step runs, and times nothing.
    if !std::env::args().any(|a| a == "--bench") {
        for (k, v) in &files {
            black_box(syntax::parse(k, v));
        }
        black_box(ws.resolve(today));
        println!("bench: {FILES} files, {lines} lines; run with cargo bench to time them");
        return;
    }

    let parse = median(|| {
        for (k, v) in &files {
            black_box(syntax::parse(k, v));
        }
    });
    let resolve = median(|| {
        black_box(ws.resolve(today));
    });
    // What `sprig check` does after reading the files.
    let check = median(|| {
        let r = ws.resolve(today);
        black_box(
            r.diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        );
    });
    let now = [
        ("parse_ms", parse),
        ("resolve_ms", resolve),
        ("check_ms", check),
    ];
    println!("bench: {FILES} files, {lines} lines, median of {RUNS} runs");
    for (k, v) in now {
        println!("  {k:<11} {v:8.2}");
    }

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("benches/baseline.json");
    if std::env::args().any(|a| a == "--save") {
        let body: Vec<String> = now
            .iter()
            .map(|(k, v)| format!("  \"{k}\": {v:.2}"))
            .collect();
        fs::write(&path, format!("{{\n{}\n}}\n", body.join(",\n"))).unwrap();
        println!("bench: wrote {}", path.display());
        return;
    }
    let base = fs::read_to_string(&path).expect("benches/baseline.json exists");
    let mut slow = Vec::new();
    for (k, v) in now {
        let b: f64 = base
            .lines()
            .find_map(|l| l.trim().strip_prefix(&format!("\"{k}\": ")))
            .and_then(|v| v.trim_end_matches(',').parse().ok())
            .unwrap_or_else(|| panic!("baseline.json has no {k}"));
        println!("  {k:<11} baseline {b:8.2}, {:4.2}x", v / b);
        if v > b * SLOWER {
            slow.push(format!(
                "{k} {v:.2} ms is more than {SLOWER}x the baseline {b:.2} ms"
            ));
        }
    }
    if !slow.is_empty() {
        eprintln!(
            "bench: slower than the baseline allows:\n  {}",
            slow.join("\n  ")
        );
        std::process::exit(1);
    }
}
