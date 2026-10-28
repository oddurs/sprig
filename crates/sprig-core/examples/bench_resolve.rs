//! Times resolving a 5,000-line workspace in two shapes:
//!
//! - `hub`: one plan grafting 49 others, like `examples/bakery/`.
//! - `chain`: each file grafting the next, 50 deep. Every file's tree holds
//!   everything beneath it, so the output, and the time, grow with the square
//!   of the depth.
//!
//! ```sh
//! cargo run --release -p sprig-core --example bench_resolve [hub|chain]
//! ```

use std::time::Instant;

use sprig_core::{Date, Workspace};

const REPS: u32 = 5;

fn workspace(chain: bool) -> Workspace {
    let mut ws = Workspace::new();
    for i in 0..50 {
        let mut s = format!("Part {i}\n\n");
        if chain && i + 1 < 50 {
            s += &format!("+ [[part{}]]\n", i + 1);
        }
        if !chain && i == 0 {
            for j in 1..50 {
                s += &format!("+ [[part{j}]]\n");
            }
        }
        while s.lines().count() < 100 {
            let n = s.lines().count();
            s += &format!(
                "- Task {n}  ^t{n}  est:2h  @p{}  after:[[part{}^t{n}]]\n  - Step  due:2026-11-0{}\n",
                n % 7,
                (i + 1) % 50,
                n % 9 + 1
            );
        }
        ws.insert(format!("part{i}.sprig"), s);
    }
    ws
}

fn main() {
    let only = std::env::args().nth(1);
    let today = Date::parse_iso("2026-10-01").expect("a date");
    for (name, chain) in [("hub", false), ("chain", true)] {
        if only.as_deref().is_some_and(|o| o != name) {
            continue;
        }
        let ws = workspace(chain);
        let t = Instant::now();
        for _ in 0..REPS {
            std::hint::black_box(ws.resolve(today));
        }
        println!("{name}: {:?} per resolve", t.elapsed() / REPS);
    }
}
