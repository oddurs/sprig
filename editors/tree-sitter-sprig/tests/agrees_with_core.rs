//! The grammar reads every line the way the core does: for every input in
//! the conformance suite and the examples, each line has the same kind, and
//! each item, answer and title the same tokens, as `sprig_core::syntax::lex`.

use std::fs;
use std::path::{Path, PathBuf};

use sprig_core::syntax::{self, LineKind};
use sprig_core::token::Piece;

fn inputs() -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "sprig") {
                out.push(p);
            }
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut out = Vec::new();
    walk(&root.join("tests/conformance"), &mut out);
    walk(&root.join("examples"), &mut out);
    out.sort();
    out
}

/// What one line is, and its tokens as `kind:text`.
type Line = (&'static str, Vec<String>);

fn core(src: &str) -> Vec<Line> {
    syntax::lex(src)
        .iter()
        .map(|l| {
            let kind = match l.kind {
                LineKind::Blank => "blank",
                LineKind::Title => "title",
                LineKind::Comment => "comment",
                LineKind::Note => "note",
                LineKind::Escaped => "escaped",
                LineKind::Item(_) => "item",
                LineKind::Answer => "answer",
            };
            let mut toks = Vec::new();
            let mut pieces = l.pieces.iter().peekable();
            while let Some((p, s)) = pieces.next() {
                let text = &src[s.start..s.end];
                match p {
                    Piece::Key => {
                        // A field is one token in the grammar: its key and value.
                        let end = match pieces.peek() {
                            Some((Piece::Value, v)) => v.end,
                            _ => s.end,
                        };
                        if text.eq_ignore_ascii_case("after:") {
                            // after: is its own token, compared by what it names below.
                        } else {
                            toks.push(format!("field:{}", &src[s.start..end]));
                        }
                    }
                    Piece::Person | Piece::Tag | Piece::Anchor | Piece::Priority => {
                        toks.push(format!("{}:{text}", p.as_str()))
                    }
                    Piece::Value | Piece::Link => {}
                }
            }
            (kind, toks)
        })
        .collect()
}

fn grammar(src: &str) -> Vec<Line> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_sprig::LANGUAGE.into())
        .unwrap();
    let tree = parser.parse(src, None).unwrap();
    let rows = src.split('\n').count();
    let mut out: Vec<Line> = vec![("blank", Vec::new()); rows];
    let root = tree.root_node();
    let mut cursor = root.walk();
    for node in root.named_children(&mut cursor) {
        let row = node.start_position().row;
        let kind = match node.kind() {
            "title" => "title",
            "comment" => "comment",
            "note" => "note",
            "escaped_note" => "escaped",
            "item" => "item",
            "answer" => "answer",
            other => panic!("unexpected top-level node {other}"),
        };
        let mut toks = Vec::new();
        let mut c = node.walk();
        for t in node.named_children(&mut c) {
            let text = &src[t.byte_range()];
            match t.kind() {
                "person" | "tag" | "anchor" | "priority" | "field" => {
                    toks.push(format!("{}:{text}", t.kind()))
                }
                // The core shows an after: value as the anchor it names.
                "after" => {
                    let value = &text[text.find(':').unwrap() + 1..];
                    if let Some(name) = value.strip_prefix('^')
                        && !name.is_empty()
                        && name
                            .chars()
                            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                    {
                        toks.push(format!("anchor:{value}"));
                    }
                }
                _ => {}
            }
        }
        out[row] = (kind, toks);
    }
    out
}

#[test]
fn every_line_has_the_kind_and_tokens_the_core_gives_it() {
    let inputs = inputs();
    assert!(inputs.len() > 200, "the inputs were not found");
    let mut wrong = Vec::new();
    for path in &inputs {
        let src = fs::read_to_string(path).unwrap();
        let (want, got) = (core(&src), grammar(&src));
        for (i, (w, g)) in want.iter().zip(&got).enumerate() {
            let tokens_matter = matches!(w.0, "item" | "answer" | "title");
            if w.0 != g.0 || (tokens_matter && w.1 != g.1) {
                wrong.push(format!(
                    "{}:{}: core {w:?}, grammar {g:?}",
                    path.display(),
                    i + 1
                ));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "{} lines differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

#[test]
fn no_input_parses_with_an_error() {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_sprig::LANGUAGE.into())
        .unwrap();
    for path in inputs() {
        let src = fs::read_to_string(&path).unwrap();
        let tree = parser.parse(&src, None).unwrap();
        assert!(
            !tree.root_node().has_error(),
            "{}: {}",
            path.display(),
            tree.root_node().to_sexp()
        );
    }
}

#[test]
fn the_highlight_query_compiles() {
    tree_sitter::Query::new(
        &tree_sitter_sprig::LANGUAGE.into(),
        tree_sitter_sprig::HIGHLIGHTS_QUERY,
    )
    .expect("queries/highlights.scm is valid for the grammar");
}
