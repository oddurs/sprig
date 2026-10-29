#![deny(missing_docs)]
//! The Sprig parser and resolver.
//!
//! This crate takes source text and a date and returns answers. It never
//! touches the filesystem or reads the clock, so the same input gives the same
//! output on every machine and every day; `scripts/task lint` holds it to that.
//!
//! What stays stable across releases, for the crate, the format and the
//! command line, is in `docs/stability.md` in the repository.
//!
//! # Example
//!
//! ```
//! use sprig_core::{Date, Workspace};
//!
//! let mut ws = Workspace::new();
//! ws.insert("plan.sprig", "Launch  @kim\n\n- Logo  ^logo\n- Menus  after:^logo\n");
//! let res = ws.resolve(Date::new(2026, 10, 1).unwrap());
//!
//! let tree = res.tree("plan.sprig").unwrap();
//! assert_eq!(tree.root.label, "Launch");
//! assert_eq!(tree.root.progress.total, 2);
//! // Menus waits on the logo, and Kim owns both.
//! assert!(tree.root.children[1].blocked);
//! assert_eq!(tree.root.children[1].people, ["kim"]);
//! assert!(res.diagnostics.is_empty());
//! ```

pub mod agenda;
pub mod date;
pub mod diagnostic;
pub mod edit;
pub mod export;
pub mod graph;
pub mod import;
pub mod json;
pub mod next;
pub mod query;
pub mod refs;
pub mod syntax;
pub mod token;
pub mod workspace;

// The README's example is a test too, so the first code a reader sees runs.
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

pub use date::Date;
pub use diagnostic::{Code, Diagnostic, Severity, Span};
pub use syntax::{Document, Mark, Node, Parsed, parse};
pub use workspace::{Resolution, Resolved, Status, Tree, Wait, Workspace};
