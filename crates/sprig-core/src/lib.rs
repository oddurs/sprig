//! The Sprig parser and resolver.
//!
//! This crate takes source text and a date and returns answers. It never
//! touches the filesystem or reads the clock, so the same input gives the same
//! output on every machine and every day; `scripts/task lint` holds it to that.
