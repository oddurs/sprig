# Editors

Support for Sprig in editors. Highlighting comes from a grammar; everything
else (problems as you type, completion, navigation, hover, code actions and
formatting) comes from the language server, `sprig lsp`, so every editor gets
the same answers.

| Folder or file | For |
| --- | --- |
| [`sprig.tmLanguage.json`](sprig.tmLanguage.json) | A TextMate grammar: VS Code, and anything else that reads one. Its snapshots are in [`tests/textmate/`](tests/textmate). |
| [`tree-sitter-sprig/`](tree-sitter-sprig) | A tree-sitter grammar with highlight queries: Neovim, Helix and Zed. |
| [`vscode/`](vscode) | The VS Code extension, with `sprig lsp` bundled. |
| [`neovim/`](neovim) | A single Lua file that sets up highlighting and the language server, with nothing else installed. |

Setup for each editor is in the
[editor guide](https://oddurs.github.io/sprig/docs/editors/).

```sh
scripts/task editors    # grammar snapshots, the tree-sitter corpus, and the extension in a real VS Code
```

The grammars only highlight. What a file means is decided by `sprig-core`, and
`tree-sitter-sprig/tests/agrees_with_core.rs` fails if the tree-sitter grammar
reads a line differently.
