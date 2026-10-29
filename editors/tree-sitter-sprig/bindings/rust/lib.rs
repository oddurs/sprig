//! The Sprig grammar for [tree-sitter](https://tree-sitter.github.io/), with
//! its highlight queries.
//!
//! ```
//! let mut parser = tree_sitter::Parser::new();
//! parser
//!     .set_language(&tree_sitter_sprig::LANGUAGE.into())
//!     .expect("the grammar loads");
//! let tree = parser.parse("Groceries\n\n- milk\n", None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```

use tree_sitter_language::LanguageFn;

unsafe extern "C" {
    fn tree_sitter_sprig() -> *const ();
}

/// The grammar.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_sprig) };

/// Highlight queries, with the capture names nvim-treesitter and Helix use.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../../queries/highlights.scm");

/// What the parser knows about each node type.
pub const NODE_TYPES: &str = include_str!("../../src/node-types.json");

// The README's example is a test too, so the first code a reader sees runs.
#[doc = include_str!("../../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
