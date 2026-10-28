//! The Sprig parser and resolver.
//!
//! This crate takes source text and a date and returns answers. It never
//! touches the filesystem or reads the clock, so the same input gives the same
//! output on every machine and every day; `scripts/task lint` holds it to that.

pub mod date;
pub mod diagnostic;
pub mod edit;
pub mod json;
pub mod next;
pub mod refs;
pub mod syntax;
pub mod token;
pub mod workspace;

pub use date::Date;
pub use diagnostic::{Code, Diagnostic, Severity, Span};
pub use syntax::{Document, Mark, Node, Parsed, parse};
pub use workspace::{Resolution, Resolved, Status, Tree, Wait, Workspace};
