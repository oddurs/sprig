//! Queries for `sprig find`: words and filters, all of which an item must
//! match (spec/tools.md T5).
//!
//! A query is split on whitespace. `@who` and `#tag` match the item's people
//! (after §6.1) and tags, `is:` its state, and `due:` its own due date; any
//! other word must appear in its text, ignoring case. A word that looks like a
//! filter but is not one is an error, so a typo never quietly matches
//! nothing.
//!
//! # Example
//!
//! ```
//! use sprig_core::{Date, Workspace, query};
//!
//! let mut ws = Workspace::new();
//! ws.insert("plan.sprig", "Plan\n\n- Logo  @kim  due:2026-10-02\n- Menus  @teo\n");
//! let today = Date::new(2026, 10, 1).unwrap();
//! let res = ws.resolve(today);
//!
//! let q = query::parse("@kim due:<=fri", today).unwrap();
//! let hits = query::find(&res, &q);
//! assert_eq!(hits.len(), 1);
//! assert!(q.matches(hits[0].node));
//! assert!(query::parse("colour:red", today).is_err());
//! ```

use std::fmt;

use crate::date::{self, Date};
use crate::workspace::{Resolution, Resolved, Status};

/// A state `is:` asks about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Is {
    /// To do, doing, or an unanswered question (§7.1).
    Open,
    /// Written or settled to do.
    Todo,
    /// In progress.
    Doing,
    /// Done, or an answered question.
    Done,
    /// Dropped.
    Dropped,
    /// Deferred.
    Later,
    /// An unanswered question.
    Ask,
    /// Open, and waiting on something (§7.3).
    Blocked,
}

const IS: [(&str, Is); 8] = [
    ("open", Is::Open),
    ("todo", Is::Todo),
    ("doing", Is::Doing),
    ("done", Is::Done),
    ("dropped", Is::Dropped),
    ("later", Is::Later),
    ("ask", Is::Ask),
    ("blocked", Is::Blocked),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cmp {
    Before,
    OnOrBefore,
    On,
    OnOrAfter,
    After,
}

/// A parsed query.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Query {
    words: Vec<String>,
    people: Vec<String>,
    tags: Vec<String>,
    is: Vec<Is>,
    due: Vec<(Cmp, Date)>,
}

/// Why a query could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum QueryError {
    /// A `key:` term that is not a filter; the term as written.
    UnknownFilter(String),
    /// A filter whose value cannot be read, such as `due:soon`; the term as written.
    BadValue(String),
}

/// The filters a query may use, for an error message.
pub const FILTERS: &str = "@who, #tag, is:open|todo|doing|done|dropped|later|ask|blocked, \
due:DATE, due:<DATE, due:<=DATE, due:>DATE, due:>=DATE";

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::UnknownFilter(w) => {
                write!(f, "{w} is not a filter; the filters are {FILTERS}")
            }
            QueryError::BadValue(w) => {
                write!(f, "{w} is not a filter value; the filters are {FILTERS}")
            }
        }
    }
}

impl std::error::Error for QueryError {}

/// Reads a query, resolving dates against `today` the way `due:` fields are
/// read (§9.3).
pub fn parse(q: &str, today: Date) -> Result<Query, QueryError> {
    let mut out = Query::default();
    for w in q.split_whitespace() {
        if let Some(name) = w.strip_prefix('@').filter(|n| !n.is_empty()) {
            out.people.push(name.to_lowercase());
        } else if let Some(tag) = w.strip_prefix('#').filter(|n| !n.is_empty()) {
            out.tags.push(tag.to_lowercase());
        } else if let Some((key, value)) = w.split_once(':')
            && !key.is_empty()
            && key.chars().all(char::is_alphabetic)
        {
            match key.to_lowercase().as_str() {
                "is" => {
                    let is = IS
                        .iter()
                        .find(|(n, _)| n.eq_ignore_ascii_case(value))
                        .ok_or_else(|| QueryError::BadValue(w.to_string()))?;
                    out.is.push(is.1);
                }
                "due" => {
                    let (cmp, rest) = if let Some(r) = value.strip_prefix("<=") {
                        (Cmp::OnOrBefore, r)
                    } else if let Some(r) = value.strip_prefix(">=") {
                        (Cmp::OnOrAfter, r)
                    } else if let Some(r) = value.strip_prefix('<') {
                        (Cmp::Before, r)
                    } else if let Some(r) = value.strip_prefix('>') {
                        (Cmp::After, r)
                    } else {
                        (Cmp::On, value)
                    };
                    let d = date::resolve(rest, today)
                        .ok_or_else(|| QueryError::BadValue(w.to_string()))?;
                    out.due.push((cmp, d));
                }
                _ => return Err(QueryError::UnknownFilter(w.to_string())),
            }
        } else {
            out.words.push(w.to_lowercase());
        }
    }
    Ok(out)
}

impl Query {
    /// Whether an item matches every part of the query.
    pub fn matches(&self, n: &Resolved) -> bool {
        let label = n.label.to_lowercase();
        self.words.iter().all(|w| label.contains(w.as_str()))
            && self
                .people
                .iter()
                .all(|p| n.people.iter().any(|q| q.to_lowercase() == *p))
            && self
                .tags
                .iter()
                .all(|t| n.tokens.tags.iter().any(|u| u.to_lowercase() == *t))
            && self.is.iter().all(|is| match is {
                Is::Open => n.status.is_open(),
                Is::Todo => n.status == Status::Todo,
                Is::Doing => n.status == Status::Doing,
                Is::Done => n.status == Status::Done,
                Is::Dropped => n.status == Status::Dropped,
                Is::Later => n.status == Status::Later,
                Is::Ask => n.status == Status::Question,
                Is::Blocked => n.blocked,
            })
            && self.due.iter().all(|(cmp, d)| {
                n.due.is_some_and(|due| match cmp {
                    Cmp::Before => due < *d,
                    Cmp::OnOrBefore => due <= *d,
                    Cmp::On => due == *d,
                    Cmp::OnOrAfter => due >= *d,
                    Cmp::After => due > *d,
                })
            })
    }
}

/// An item that matched, with the labels of the items above it in its file.
#[derive(Clone, Debug)]
pub struct Hit<'r> {
    /// The item that matched.
    pub node: &'r Resolved,
    /// The labels from the top of its tree down to its parent.
    pub path: Vec<&'r str>,
}

/// Every item written in the workspace that matches, each once, in the
/// file it is written in, in file and document order. An item is read in
/// its own file's tree, where its status does not depend on who grafts it.
pub fn find<'r>(res: &'r Resolution, q: &Query) -> Vec<Hit<'r>> {
    fn walk<'r>(n: &'r Resolved, path: &mut Vec<&'r str>, q: &Query, out: &mut Vec<Hit<'r>>) {
        for c in n.children.iter().filter(|c| !c.grafted) {
            if q.matches(c) {
                out.push(Hit {
                    node: c,
                    path: path.clone(),
                });
            }
            path.push(&c.label);
            walk(c, path, q, out);
            path.pop();
        }
    }
    let mut out = Vec::new();
    for t in res.trees() {
        walk(&t.root, &mut Vec::new(), q, &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> Date {
        Date::parse_iso("2026-10-01").unwrap()
    }

    #[test]
    fn filters_read_their_values_and_typos_are_refused() {
        let q = parse("oven @Teo #kitchen is:open due:<fri", today()).unwrap();
        assert_eq!(q.words, ["oven"]);
        assert_eq!(q.people, ["teo"]);
        assert_eq!(q.is, [Is::Open]);
        assert_eq!(
            q.due,
            [(Cmp::Before, Date::parse_iso("2026-10-02").unwrap())]
        );
        assert_eq!(
            parse("is:opn", today()),
            Err(QueryError::BadValue("is:opn".into()))
        );
        assert_eq!(
            parse("owner:dana", today()),
            Err(QueryError::UnknownFilter("owner:dana".into()))
        );
        assert!(parse("due:soonish", today()).is_err());
        // A time is not a filter: its key is not a word.
        assert_eq!(parse("2:30", today()).unwrap().words, ["2:30"]);
    }
}
