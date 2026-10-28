//! Edits change only the bytes they mean to (cairn 0043), and `fmt` is
//! idempotent and changes nothing but layout and typed dates (cairn 0044).

use std::fs;
use std::path::{Path, PathBuf};

use proptest::prelude::*;
use serde_json::Value;
use sprig_core::edit::{self, Change, Place};
use sprig_core::{Date, Mark, Node, json, parse};

fn today() -> Date {
    Date::parse_iso("2026-10-01").unwrap()
}

/// Every `.sprig` input in the conformance suite and the examples.
fn inputs() -> Vec<(PathBuf, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut out = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
        let mut entries: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "sprig") {
                out.push((p.clone(), String::from_utf8(fs::read(&p).unwrap()).unwrap()));
            }
        }
    }
    walk(&root.join("tests/conformance"), &mut out);
    walk(&root.join("examples"), &mut out);
    assert!(out.len() > 50);
    out
}

fn items(n: &Node, out: &mut Vec<(usize, Option<Mark>)>) {
    for c in &n.children {
        out.push((c.line, c.mark));
        items(c, out);
    }
}

/// Line numbers whose bytes an edit may touch: those its replacements overlap,
/// in the old source.
fn touched_lines(src: &str, change: &Change) -> Vec<usize> {
    let starts: Vec<usize> = std::iter::once(0)
        .chain(src.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let line_of = |b: usize| starts.partition_point(|&s| s <= b);
    let mut out = Vec::new();
    for e in &change.edits {
        for l in line_of(e.span.start)..=line_of(e.span.end.max(e.span.start)) {
            out.push(l);
        }
    }
    out
}

/// The edits name every byte that changed: applying them is the new source,
/// and outside them old and new agree.
fn assert_honest(src: &str, change: &Change) {
    assert_eq!(edit::apply(src, &change.edits), change.source);
    for w in change.edits.windows(2) {
        assert!(w[0].span.end <= w[1].span.start, "edits overlap");
    }
    // Whether bytes outside the edits are unchanged is checked line by line
    // in `step`, where an insertion's place is known; a prefix-and-suffix
    // diff cannot say where repeated text was inserted.
    if change.edits.is_empty() {
        assert_eq!(src, change.source);
    }
}

#[derive(Clone, Debug)]
enum Op {
    Mark(usize, Mark),
    Tick(usize),
    Field(usize, &'static str, Option<&'static str>),
    Child(usize),
    Sibling(usize),
    Answer(usize),
}

fn op() -> impl Strategy<Value = Op> {
    let mark = prop::sample::select(vec![
        Mark::Todo,
        Mark::Doing,
        Mark::Done,
        Mark::Dropped,
        Mark::Later,
        Mark::Question,
    ]);
    let key = prop::sample::select(vec!["due", "est", "where", "after"]);
    let value = prop::sample::select(vec![
        Some("2026-11-02"),
        Some("fri"),
        Some("two words"),
        None,
    ]);
    prop_oneof![
        (any::<usize>(), mark).prop_map(|(i, m)| Op::Mark(i, m)),
        any::<usize>().prop_map(Op::Tick),
        (any::<usize>(), key, value).prop_map(|(i, k, v)| Op::Field(i, k, v)),
        any::<usize>().prop_map(Op::Child),
        any::<usize>().prop_map(Op::Sibling),
        any::<usize>().prop_map(Op::Answer),
    ]
}

/// What an edit says the parsed result will show.
type Promise = Box<dyn Fn(&Node)>;

/// Applies one operation to the `i`-th item, and checks what it promised.
fn step(src: &str, op: &Op) -> String {
    let doc = parse("t", src).document;
    let mut all = Vec::new();
    items(&doc.root, &mut all);
    if all.is_empty() {
        return src.to_string();
    }
    let pick = |i: usize| all[i % all.len()];
    let lines_before = src.split('\n').count();
    let (change, promise): (Change, Promise) = match op.clone() {
        Op::Mark(i, m) => {
            let (line, _) = pick(i);
            match edit::set_mark(src, line, m) {
                Ok(c) => (
                    c,
                    Box::new(move |root: &Node| assert_eq!(find(root, line).mark, Some(m))),
                ),
                Err(_) => return src.to_string(),
            }
        }
        Op::Tick(i) => {
            let (line, _) = pick(i);
            match edit::tick(src, line, today()) {
                Ok((c, _)) => (c, Box::new(|_: &Node| {})),
                Err(_) => return src.to_string(),
            }
        }
        Op::Field(i, k, v) => {
            let (line, _) = pick(i);
            let c = match edit::set_field(src, line, k, v) {
                Ok(c) => c,
                // Refused only where a quote already on the line would pair
                // with one the edit writes.
                Err(edit::EditError::WouldNotRead(_)) => {
                    let text = src.split('\n').nth(line - 1).unwrap_or("");
                    assert!(text.contains('"'), "refused a line with no quote: {text:?}");
                    return src.to_string();
                }
                Err(e) => panic!("{e}"),
            };
            let promise = move |root: &Node| {
                let n = find(root, line);
                match (k, v) {
                    ("after", Some(_)) | ("after", None) => {}
                    (k, Some(v)) => {
                        assert_eq!(n.tokens.field(k).map(|f| f.value.as_str()), Some(v))
                    }
                    (k, None) => assert!(n.tokens.field(k).is_none()),
                }
            };
            (c, Box::new(promise))
        }
        Op::Child(i) | Op::Sibling(i) => {
            let (line, _) = pick(i);
            let place = if matches!(op, Op::Child(_)) {
                Place::Child
            } else {
                Place::Sibling
            };
            let c = edit::add_item(src, line, place, "- added  @prop").unwrap();
            let promise = move |root: &Node| {
                let n = find(root, line);
                if place == Place::Child {
                    assert_eq!(
                        n.children.last().map(|c| c.tokens.text.as_str()),
                        Some("added")
                    );
                }
            };
            (c, Box::new(promise))
        }
        Op::Answer(i) => {
            let (line, _) = pick(i);
            let before = find(&doc.root, line).answers.len();
            let c = edit::add_answer(src, line, "agreed").unwrap();
            (
                c,
                Box::new(move |root: &Node| assert_eq!(find(root, line).answers.len(), before + 1)),
            )
        }
    };
    assert_honest(src, &change);
    // Bytes on lines no edit touched are unchanged: compare line by line,
    // allowing for lines an insertion added.
    let touched = touched_lines(src, &change);
    let old: Vec<&str> = src.split('\n').collect();
    let new: Vec<&str> = change.source.split('\n').collect();
    let added = new.len() as i64 - lines_before as i64;
    for (i, l) in old.iter().enumerate() {
        let n = i + 1;
        if touched.contains(&n) {
            continue;
        }
        let first = touched.iter().min().copied().unwrap_or(usize::MAX);
        let j = if n < first {
            i
        } else {
            (i as i64 + added) as usize
        };
        assert_eq!(
            Some(l),
            new.get(j),
            "line {n} changed though no edit touched it"
        );
    }
    promise(&parse("t", &change.source).document.root);
    change.source
}

fn find(root: &Node, line: usize) -> &Node {
    let mut hit = None;
    fn walk<'a>(n: &'a Node, line: usize, hit: &mut Option<&'a Node>) {
        for c in &n.children {
            if c.line == line {
                *hit = Some(c);
            }
            walk(c, line, hit);
        }
    }
    walk(root, line, &mut hit);
    hit.unwrap_or_else(|| panic!("no item on line {line}"))
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 1_000, ..ProptestConfig::default() })]

    #[test]
    fn random_edit_sequences_touch_only_the_lines_they_edit(
        pick in any::<prop::sample::Index>(),
        ops in prop::collection::vec(op(), 1..6),
    ) {
        let inputs = inputs();
        let (_, src) = &inputs[pick.index(inputs.len())];
        let mut src = src.clone();
        for o in &ops {
            src = step(&src, o);
        }
    }
}

/// The syntax tree, with spans dropped and date fields read against today,
/// so that formatting may move indentation and write dates out.
fn meaning(src: &str) -> Value {
    fn strip(v: &mut Value) {
        match v {
            Value::Object(m) => {
                m.remove("span");
                if let Some(Value::Object(f)) = m.get_mut("fields") {
                    for k in ["due", "start"] {
                        if let Some(Value::String(s)) = f.get_mut(k)
                            && let Some(d) = sprig_core::date::resolve(s, today())
                        {
                            *s = d.to_string();
                        }
                    }
                }
                m.values_mut().for_each(strip);
            }
            Value::Array(a) => a.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut v = json::syntax(&parse("t", src).document);
    strip(&mut v);
    v
}

#[test]
fn fmt_is_idempotent_and_keeps_the_tree_on_every_input() {
    for (path, src) in inputs() {
        let once = edit::format(&src, today());
        assert_honest(&src, &once);
        let twice = edit::format(&once.source, today());
        assert!(
            twice.is_empty(),
            "{}: formatting twice changed it again",
            path.display()
        );
        assert_eq!(
            meaning(&once.source),
            meaning(&src),
            "{}: formatting changed the tree",
            path.display()
        );
        let parsed = parse("t", &once.source);
        assert_eq!(
            parsed
                .diagnostics
                .iter()
                .filter(|d| d.code.as_str() == "S0001")
                .count(),
            parse("t", &src)
                .diagnostics
                .iter()
                .filter(|d| d.code.as_str() == "S0001")
                .count(),
        );
    }
}

fn sprig_like() -> impl Strategy<Value = String> {
    let pieces = prop::sample::select(vec![
        "- ",
        "~ ",
        "x ",
        "? ",
        "= ",
        "# ",
        "+ [[b]]",
        "\\- ",
        "// c",
        "note",
        "\n",
        "\n  ",
        "\n    ",
        "\n\t",
        "\r\n",
        "  ",
        "due:fri",
        "due:2026-10-02",
        "start:+2w",
        "@jo",
        "^a",
        "after:^a",
        "\u{feff}",
        "é",
    ]);
    prop::collection::vec(pieces, 0..60).prop_map(|v| v.concat())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 3_000, ..ProptestConfig::default() })]

    #[test]
    fn fmt_is_idempotent_and_keeps_the_tree_on_arbitrary_plans(src in sprig_like()) {
        let once = edit::format(&src, today());
        assert_honest(&src, &once);
        prop_assert!(edit::format(&once.source, today()).is_empty(), "not idempotent: {:?} -> {:?}", src, once.source);
        prop_assert_eq!(meaning(&once.source), meaning(&src), "tree changed: {:?} -> {:?}", src, once.source);
    }
}

#[test]
fn a_field_that_would_pair_with_an_unclosed_quote_is_refused() {
    // The new quote would close the stray one and swallow the field (§4.1).
    let src = "Plan\n\n- Visit  where:\"41 Mill Street  @dana\n";
    assert_eq!(
        edit::set_field(src, 3, "due", Some("two words")),
        Err(edit::EditError::WouldNotRead(3))
    );
    // A value that needs no quotes still reads back, so it is written.
    let c = edit::set_field(src, 3, "due", Some("2026-10-02")).unwrap();
    assert!(c.source.contains("@dana  due:2026-10-02"));
}
