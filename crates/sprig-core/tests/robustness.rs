//! Plans arrive pasted from everywhere. These tests hold the parser to two
//! promises: no input makes it panic, and the accidents of where a file was
//! written (CRLF, a BOM, tabs) do not change what it means.

use std::fs;
use std::path::{Path, PathBuf};

use proptest::prelude::*;
use serde_json::Value;
use sprig_core::{Date, Node, Workspace, json, parse};

fn lines_examples() -> Vec<(PathBuf, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance/lines");
    let mut out: Vec<(PathBuf, String)> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path().join("main.sprig"))
        .map(|p| {
            let src = String::from_utf8(fs::read(&p).unwrap()).unwrap();
            (p, src)
        })
        .collect();
    out.sort();
    assert!(out.len() >= 40);
    out
}

/// The syntax tree without byte offsets, which a BOM or a CR shifts.
fn shape(src: &str) -> Value {
    fn strip(v: &mut Value) {
        match v {
            Value::Object(m) => {
                m.remove("span");
                m.values_mut().for_each(strip);
            }
            Value::Array(a) => a.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut v = json::syntax(&parse("main.sprig", src).document);
    strip(&mut v);
    v
}

fn without_bom(src: &str) -> &str {
    src.strip_prefix('\u{feff}').unwrap_or(src)
}

#[test]
fn every_span_slices_back_to_its_line() {
    fn check(src: &str, lines: &[&str], n: &Node, path: &Path) {
        if let Some(s) = n.span {
            let text = &src[s.start..s.end];
            assert_eq!(
                text,
                lines[n.line - 1].trim(),
                "{}: line {}",
                path.display(),
                n.line
            );
            if let Some(m) = n.mark {
                assert!(text.starts_with(m.as_char()));
            }
        }
        for a in &n.answers {
            assert_eq!(&src[a.span.start..a.span.end], lines[a.line - 1].trim());
        }
        for p in &n.notes {
            for l in &p.lines {
                let text = &src[l.span.start..l.span.end];
                assert!(
                    text.ends_with(l.text.as_str()),
                    "{}: note {text:?}",
                    path.display()
                );
            }
        }
        for c in &n.children {
            check(src, lines, c, path);
        }
    }
    for (path, src) in lines_examples() {
        let doc = parse("main.sprig", &src).document;
        let body = without_bom(&src);
        let lines: Vec<&str> = body.split('\n').collect();
        // Offsets are into the whole file, BOM included.
        let lines: Vec<&str> = lines
            .iter()
            .map(|l| l.trim_start_matches('\u{feff}'))
            .collect();
        check(&src, &lines, &doc.root, &path);
    }
}

#[test]
fn crlf_and_bom_variants_read_the_same() {
    for (path, src) in lines_examples() {
        let clean = without_bom(&src).replace("\r\n", "\n");
        let want = shape(&clean);
        let crlf = clean.replace('\n', "\r\n");
        assert_eq!(shape(&crlf), want, "{}: CRLF", path.display());
        let bom = format!("\u{feff}{clean}");
        assert_eq!(shape(&bom), want, "{}: BOM", path.display());
        assert_eq!(
            shape(&format!("\u{feff}{crlf}")),
            want,
            "{}: BOM and CRLF",
            path.display()
        );
    }
}

#[test]
fn a_leading_tab_reads_as_two_spaces() {
    for (path, src) in lines_examples() {
        let clean = without_bom(&src).replace("\r\n", "\n");
        if clean.contains('\t') {
            continue;
        }
        let tabbed: Vec<String> = clean
            .split('\n')
            .map(|l| {
                let n = l.len() - l.trim_start_matches(' ').len();
                format!("{}{}{}", "\t".repeat(n / 2), " ".repeat(n % 2), &l[n..])
            })
            .collect();
        assert_eq!(
            shape(&tabbed.join("\n")),
            shape(&clean),
            "{}",
            path.display()
        );
    }
}

fn resolve_all(a: &str, b: &str) {
    let mut w = Workspace::new();
    w.insert("main.sprig", a);
    w.insert("b.sprig", b);
    let _ = w.wanted();
    let r = w.resolve(Date::parse_iso("2026-10-01").unwrap());
    for d in &r.diagnostics {
        let _ = d.to_string();
    }
    let _ = json::tree_document(&r, "main.sprig", Date::parse_iso("2026-10-01").unwrap());
}

// Characters that mean something to a Sprig line, so random text built from
// them reaches the interesting paths far more often than uniform Unicode.
const ALPHABET: &[&str] = &[
    "-",
    "~",
    "x",
    "/",
    ">",
    "?",
    "#",
    "+",
    "=",
    "\\",
    "//",
    " ",
    "  ",
    "\t",
    "\n",
    "\r\n",
    "\r",
    "@",
    "^",
    ":",
    "!",
    "\"",
    "[[",
    "]]",
    "[[b]]",
    "[[b^a]]",
    "[[main]]",
    "after:",
    "due:",
    "start:",
    "est:",
    "every:",
    "2026-10-02",
    "a",
    "Z",
    "é",
    "\u{feff}",
    "\u{a0}",
    "☃",
    "0.5d",
];

fn sprig_like() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(ALPHABET), 0..80).prop_map(|v| v.concat())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 10_000, ..ProptestConfig::default() })]

    #[test]
    fn arbitrary_text_never_panics(a in any::<String>(), b in sprig_like()) {
        resolve_all(&a, &b);
        resolve_all(&b, &a);
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 2_000, ..ProptestConfig::default() })]

    #[test]
    fn mutated_examples_never_panic(
        pick in any::<prop::sample::Index>(),
        edits in prop::collection::vec((any::<prop::sample::Index>(), 0usize..4, prop::sample::select(ALPHABET)), 1..8),
    ) {
        let examples = lines_examples();
        let (_, src) = &examples[pick.index(examples.len())];
        let mut s: Vec<char> = src.chars().collect();
        for (at, cut, insert) in edits {
            let i = if s.is_empty() { 0 } else { at.index(s.len() + 1) };
            let j = (i + cut).min(s.len());
            s.splice(i..j, insert.chars());
        }
        let s: String = s.into_iter().collect();
        resolve_all(&s, src);
    }
}
