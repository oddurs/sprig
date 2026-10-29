//! Naming one item from outside a file: the refs a command line, an editor or
//! an agent passes to say which item to tick or explain.
//!
//! This is tool behaviour, not format; `spec/tools.md` (T1) states it so that other
//! implementations can match it. Anchors are the stable form, the way
//! Taskwarrior's UUIDs are and its working-set numbers are not: a line number
//! moves when a line is added above it. Refs name the item where it is
//! written, never a grafted copy, so a ref always points at the line an edit
//! would change.
//!
//! # Example
//!
//! ```
//! use sprig_core::{Date, Workspace, refs};
//!
//! let mut ws = Workspace::new();
//! ws.insert("lease.sprig", "Lease\n\n- Negotiate\n- Sign the lease  ^signed\n");
//! let res = ws.resolve(Date::new(2026, 10, 1).unwrap());
//!
//! // By anchor, by file and line, or by words that match one item.
//! assert_eq!(refs::resolve(&res, "^signed").unwrap().line, 4);
//! assert_eq!(refs::resolve(&res, "lease.sprig:3").unwrap().text, "Negotiate");
//! assert_eq!(refs::resolve(&res, "negot").unwrap().line, 3);
//! assert!(refs::resolve(&res, "^nowhere").is_err());
//! ```

use std::fmt;

use crate::syntax::{self, Node};
use crate::workspace::Resolution;

/// An item, where it is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// The workspace path of the file that writes the item.
    pub file: String,
    /// 1-based.
    pub line: usize,
    /// The item's text (§4.9), as candidates are listed.
    pub text: String,
}

impl fmt::Display for Item {
    /// `file:line  text`, the form candidates are listed in.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}  {}", self.file, self.line, self.text)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Why a ref named no single item.
#[non_exhaustive]
pub enum RefError {
    /// Nothing matches; the message says what was looked for.
    NotFound(String),
    /// More than one item matches: every one of them.
    Ambiguous(String, Vec<Item>),
}

impl fmt::Display for RefError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RefError::NotFound(why) => f.write_str(why),
            RefError::Ambiguous(r, items) => {
                write!(f, "{r} names {} items:", items.len())?;
                for i in items {
                    write!(f, "\n  {i}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for RefError {}

/// Every item each file writes, in path and then document order.
fn written(res: &Resolution) -> Vec<(&str, &Node)> {
    let mut out = Vec::new();
    for (file, p) in &res.parsed {
        syntax::walk(&p.document.root, &mut |n: &Node| {
            out.push((file.as_str(), n))
        });
    }
    out
}

fn item(file: &str, n: &Node) -> Item {
    Item {
        file: file.to_string(),
        line: n.line,
        text: n.tokens.text.clone(),
    }
}

fn one(r: &str, found: Vec<Item>, none: impl FnOnce() -> String) -> Result<Item, RefError> {
    match found.len() {
        0 => Err(RefError::NotFound(none())),
        1 => Ok(found.into_iter().next().expect("one")),
        _ => Err(RefError::Ambiguous(r.to_string(), found)),
    }
}

/// The workspace file a ref's file part names: the path as written, with
/// `.sprig` added if missing, or failing that a file of that name in any
/// directory, when there is exactly one.
fn file<'a>(res: &'a Resolution, name: &str) -> Result<&'a str, RefError> {
    let name = name.strip_prefix("./").unwrap_or(name).replace('\\', "/");
    let key = if name.ends_with(".sprig") {
        name.clone()
    } else {
        format!("{name}.sprig")
    };
    if let Some((k, _)) = res.parsed.get_key_value(&key) {
        return Ok(k);
    }
    let matches: Vec<&str> = res
        .parsed
        .keys()
        .map(String::as_str)
        .filter(|k| k.rsplit('/').next() == Some(key.as_str()))
        .collect();
    match matches.as_slice() {
        [one] => Ok(one),
        [] => Err(RefError::NotFound(format!("no file named {key}"))),
        many => Err(RefError::Ambiguous(
            name,
            many.iter()
                .map(|f| Item {
                    file: f.to_string(),
                    line: 0,
                    text: String::new(),
                })
                .collect(),
        )),
    }
}

fn is_anchor(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

/// Resolves a ref, trying its forms from exact to forgiving:
///
/// 1. `^anchor`: an anchor, in any file;
/// 2. `file^anchor`: an anchor in one file;
/// 3. `file:12` or `file.sprig:12`: the item on that line;
/// 4. anything else: words from an item's text, matching one item.
pub fn resolve(res: &Resolution, r: &str) -> Result<Item, RefError> {
    let r = r.trim();
    if r.is_empty() {
        return Err(RefError::NotFound("an empty ref names nothing".into()));
    }
    let all = written(res);

    if let Some(anchor) = r.strip_prefix('^')
        && is_anchor(anchor)
    {
        // The first item in each file with the anchor, as references reach
        // (Appendix B.4).
        let mut found: Vec<Item> = Vec::new();
        for (f, n) in &all {
            if n.tokens.anchor().is_some_and(|a| a.name == anchor)
                && !found.iter().any(|i| i.file == *f)
            {
                found.push(item(f, n));
            }
        }
        return one(r, found, || format!("no item has the anchor ^{anchor}"));
    }

    if let Some((name, anchor)) = r.rsplit_once('^')
        && is_anchor(anchor)
        && !name.contains(char::is_whitespace)
    {
        let f = file(res, name)?;
        let found = all
            .iter()
            .find(|(file, n)| *file == f && n.tokens.anchor().is_some_and(|a| a.name == anchor))
            .map(|(file, n)| item(file, n));
        return found.ok_or_else(|| RefError::NotFound(format!("no item ^{anchor} in {f}")));
    }

    if let Some((name, line)) = r.rsplit_once(':')
        && !line.is_empty()
        && line.bytes().all(|b| b.is_ascii_digit())
        && !name.contains(char::is_whitespace)
    {
        let f = file(res, name)?;
        let line: usize = line
            .parse()
            .map_err(|_| RefError::NotFound(format!("{r}: no such line")))?;
        return all
            .iter()
            .find(|(file, n)| *file == f && n.line == line)
            .map(|(file, n)| item(file, n))
            .ok_or_else(|| RefError::NotFound(format!("line {line} of {f} is not an item")));
    }

    // Words: a case-insensitive part of an item's text. Notes and answers
    // are never searched. An exact match wins over partial ones.
    let want = r.to_lowercase();
    let partial: Vec<Item> = all
        .iter()
        .filter(|(_, n)| n.tokens.text.to_lowercase().contains(&want))
        .map(|(f, n)| item(f, n))
        .collect();
    let exact: Vec<Item> = partial
        .iter()
        .filter(|i| i.text.to_lowercase() == want)
        .cloned()
        .collect();
    if exact.len() == 1 {
        return Ok(exact.into_iter().next().expect("one"));
    }
    one(r, partial, || format!("no item's text contains {r:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Date, Workspace};

    fn bakery() -> Resolution {
        // Compiled in, so the core's tests read no files either.
        macro_rules! file {
            ($name:literal) => {
                (
                    $name,
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../../examples/bakery/",
                        $name
                    )),
                )
            };
        }
        let mut w = Workspace::new();
        for (name, src) in [
            file!("bakery.sprig"),
            file!("dana.sprig"),
            file!("groceries.sprig"),
            file!("kitchen.sprig"),
            file!("launch.sprig"),
            file!("lease.sprig"),
        ] {
            w.insert(name, src);
        }
        w.resolve(Date::parse_iso("2026-10-01").unwrap())
    }

    fn at(r: &str) -> String {
        let i = resolve(&bakery(), r).unwrap_or_else(|e| panic!("{r}: {e}"));
        format!("{}:{}", i.file, i.line)
    }

    #[test]
    fn an_anchor_is_found_in_whichever_file_writes_it() {
        assert_eq!(at("^signed"), "lease.sprig:13");
        assert_eq!(at("^oven"), "kitchen.sprig:9");
    }

    #[test]
    fn an_anchor_in_one_file() {
        assert_eq!(at("lease^signed"), "lease.sprig:13");
        assert_eq!(at("lease.sprig^terms"), "lease.sprig:6");
        assert!(matches!(
            resolve(&bakery(), "kitchen^signed"),
            Err(RefError::NotFound(_))
        ));
        assert!(matches!(
            resolve(&bakery(), "nowhere^x"),
            Err(RefError::NotFound(_))
        ));
    }

    #[test]
    fn a_line_as_compilers_print_it() {
        assert_eq!(at("lease.sprig:13"), "lease.sprig:13");
        assert_eq!(at("lease:13"), "lease.sprig:13");
        let e = resolve(&bakery(), "lease.sprig:4").unwrap_err();
        assert_eq!(e.to_string(), "line 4 of lease.sprig is not an item");
    }

    #[test]
    fn words_match_one_items_text_and_never_notes() {
        assert_eq!(at("grease trap"), "lease.sprig:9");
        assert_eq!(at("GREASE TRAP"), "lease.sprig:9");
        // "Mill Street" is in a note in lease.sprig and in the name question;
        // only the question is an item.
        assert_eq!(at("mill street"), "bakery.sprig:22");
        // A note's words alone match nothing.
        assert!(matches!(
            resolve(&bakery(), "three-phase power"),
            Err(RefError::NotFound(_))
        ));
    }

    #[test]
    fn a_ref_into_a_grafted_branch_names_the_line_that_writes_it() {
        // dana.sprig grafts kitchen^oven; the quote is written in kitchen.sprig.
        assert_eq!(at("third quote"), "kitchen.sprig:11");
    }

    #[test]
    fn an_ambiguous_ref_lists_every_candidate() {
        let e = resolve(&bakery(), "sack").unwrap_err();
        let RefError::Ambiguous(_, items) = &e else {
            panic!("{e}")
        };
        assert_eq!(items.len(), 3, "rye, strong white and spelt");
        assert!(e.to_string().contains("\n  "));
        // An exact match wins over partial ones.
        assert_eq!(at("design"), "kitchen.sprig:3");
    }

    #[test]
    fn an_anchor_in_two_files_is_ambiguous() {
        let mut w = Workspace::new();
        w.insert("a.sprig", "A\n\n- One  ^x\n");
        w.insert("b.sprig", "B\n\n- Two  ^x\n");
        let r = w.resolve(Date::parse_iso("2026-10-01").unwrap());
        let e = resolve(&r, "^x").unwrap_err();
        assert_eq!(
            e.to_string(),
            "^x names 2 items:\n  a.sprig:3  One\n  b.sprig:3  Two"
        );
        assert_eq!(resolve(&r, "b^x").unwrap().text, "Two");
    }

    #[test]
    fn a_file_may_be_named_by_its_name_alone_when_that_is_unique() {
        let mut w = Workspace::new();
        w.insert("rooms/kitchen.sprig", "K\n\n- Oven  ^oven\n");
        let r = w.resolve(Date::parse_iso("2026-10-01").unwrap());
        assert_eq!(
            resolve(&r, "kitchen^oven").unwrap().file,
            "rooms/kitchen.sprig"
        );
        assert_eq!(resolve(&r, "rooms/kitchen:3").unwrap().line, 3);
    }
}
