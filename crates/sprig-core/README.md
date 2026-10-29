# sprig-core

The parser, resolver and edit API for [Sprig](https://oddurs.github.io/sprig/),
a plain-text format for plans. It is the one implementation of the format:
the `sprig` command, the language server, the MCP server and the website all
call it.

```rust
use sprig_core::{Date, Workspace};

let mut ws = Workspace::new();
ws.insert("plan.sprig", "Launch  @kim\n\n- Logo  ^logo\n- Menus  after:^logo\n");
let res = ws.resolve(Date::new(2026, 10, 1).unwrap());

let menus = &res.tree("plan.sprig").unwrap().root.children[1];
assert!(menus.blocked);
assert_eq!(menus.people, ["kim"]);
assert!(res.diagnostics.is_empty());
```

## What it promises

- **No I/O.** It never reads a file, the environment, the network or the
  clock. You pass the text of each file and today's date, so an answer is the
  same on every machine and every day, and the crate compiles to WebAssembly
  unchanged. `Workspace::wanted` says which files a plan names that you have
  not given it yet.
- **No panics on input.** Every problem in a file becomes a `Diagnostic` with
  a stable code from the
  [catalogue](https://github.com/oddurs/sprig/blob/main/spec/diagnostics.md).
- **Edits keep every byte they don't mean to change.** `edit::tick`,
  `edit::set_mark`, `edit::format` and the rest return the new text and the
  exact replacements that made it.
- **Grafts never leave the workspace root.** A path in a plan is untrusted
  input.

It is held to the [specification](https://github.com/oddurs/sprig/blob/main/spec/sprig.md)
by the [conformance suite](https://github.com/oddurs/sprig/tree/main/tests/conformance),
and what stays stable across releases is in
[`docs/stability.md`](https://github.com/oddurs/sprig/blob/main/docs/stability.md).

## License

MIT.
