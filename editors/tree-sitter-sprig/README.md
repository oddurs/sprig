# tree-sitter-sprig

A [tree-sitter](https://tree-sitter.github.io/) grammar for
[Sprig](https://oddurs.github.io/sprig/), a plain-text format for plans, with
highlight queries that use the capture names nvim-treesitter and Helix share.

```rust
let mut parser = tree_sitter::Parser::new();
parser
    .set_language(&tree_sitter_sprig::LANGUAGE.into())
    .expect("the grammar loads");
let tree = parser.parse("Groceries\n\n- milk\n", None).unwrap();
assert!(!tree.root_node().has_error());
```

The grammar highlights; it does not decide what a file means. That is
`sprig-core`'s job, and `tests/agrees_with_core.rs` checks that every line of
every conformance example gets the same kind and the same tokens from both.

## Working on it

`grammar.js` is the source, and `src/scanner.c` ends a line at a newline or at
the end of the file. `src/parser.c` is generated, and committed so that a
build needs no tree-sitter CLI:

```sh
tree-sitter generate --abi 14    # after changing grammar.js
tree-sitter test                 # test/corpus/
scripts/task editors             # everything CI runs for the editors
```

Setup for Neovim, Helix and Zed is in the
[editor guide](https://oddurs.github.io/sprig/docs/editors/).

## License

MIT.
