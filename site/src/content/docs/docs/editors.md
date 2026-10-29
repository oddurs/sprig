---
title: Set up your editor
description: Highlighting, problems as you type, completion and ticking for .sprig files in VS Code, Neovim, Helix, Zed and any editor that speaks LSP.
---

Every editor gets the same two things from Sprig:

- **The language server**, `sprig lsp`, which comes with the `sprig` command.
  It shows problems as you type, including the ones a change in one file
  causes in another. It also completes `@people`, `^anchors`, `[[file]]` links
  and field keys, goes to the definition of an anchor or link, finds every
  line that names it, shows progress and blockers on hover, and ticks items
  and formats files.
- **A grammar** for highlighting: the TextMate grammar in
  `editors/sprig.tmLanguage.json`, or the tree-sitter grammar in
  `editors/tree-sitter-sprig`.

Install `sprig` first; the editors below find it on your `PATH`:

```sh
cargo install --locked --git https://github.com/oddurs/sprig sprig-cli
sprig --version
```

The language server treats the folder your editor opened as the workspace, so
open the folder that holds your plans rather than a single file.

## VS Code

The extension in `editors/vscode` registers `.sprig` files, highlights them
and runs the language server it bundles, so it needs nothing else installed.
It isn't on the Marketplace yet. Until it is, install a package built by CI:

1. Download `sprig-<platform>.vsix` from the latest run of the
   [vscode workflow](https://github.com/oddurs/sprig/actions/workflows/vscode.yml).
2. Run **Extensions: Install from VSIX…** from the command palette and pick it.

To use a different `sprig`, set `sprig.server.path`.

CI holds this to a test that opens `examples/bakery/` in VS Code with only the
extension installed and checks the problems it reports.

## Neovim

Neovim 0.9 or later needs no plugins. Copy
[`editors/neovim/sprig.lua`](https://github.com/oddurs/sprig/blob/main/editors/neovim/sprig.lua)
into your config, for example as `~/.config/nvim/plugin/sprig.lua`. It sets the
filetype, starts `sprig lsp` for every `.sprig` buffer, and turns on tree-sitter
highlighting when the parser is installed.

For highlighting, build the parser and copy the queries:

```sh
git clone https://github.com/oddurs/sprig
cd sprig/editors/tree-sitter-sprig
tree-sitter build -o ~/.config/nvim/parser/sprig.so
mkdir -p ~/.config/nvim/queries/sprig
cp queries/highlights.scm ~/.config/nvim/queries/sprig/
```

If you use nvim-lspconfig, the same server can be added as a custom
configuration with `cmd = { 'sprig', 'lsp' }` and `filetypes = { 'sprig' }`.

CI follows this section on a clean config: it loads `sprig.lua` and the
parser, opens `examples/bakery/lease.sprig` and deletes `^signed`. It then
waits for `bakery.sprig` and `kitchen.sprig`, which name it, to report a
problem.

## Helix

Add to `~/.config/helix/languages.toml`, with `rev` set to a commit of this
repository:

```toml
[language-server.sprig]
command = "sprig"
args = ["lsp"]

[[language]]
name = "sprig"
scope = "source.sprig"
file-types = ["sprig"]
roots = [".git"]
comment-token = "//"
indent = { tab-width = 2, unit = "  " }
language-servers = ["sprig"]

[[grammar]]
name = "sprig"
source = { git = "https://github.com/oddurs/sprig", rev = "COMMIT", subpath = "editors/tree-sitter-sprig" }
```

Then fetch and build the grammar, and copy the highlight queries into your
runtime directory:

```sh
hx --grammar fetch && hx --grammar build
mkdir -p ~/.config/helix/runtime/queries/sprig
cp editors/tree-sitter-sprig/queries/highlights.scm ~/.config/helix/runtime/queries/sprig/
```

`hx --health sprig` should then list the language server and highlighting.

## Zed

Zed reaches a language server and a grammar only through an extension, and
there isn't one for Sprig yet (cairn 0147). Until there is, Zed has no Sprig
support.

## Other editors

Any editor with an LSP client can run `sprig lsp` over standard input and
output for files ending in `.sprig`:

- **Emacs:** with eglot, add `(add-to-list 'eglot-server-programs '(sprig-mode . ("sprig" "lsp")))`.
- **Sublime Text:** with the LSP package, add a client whose command is
  `["sprig", "lsp"]` and whose selector matches `.sprig` files.
- **Anything else:** the server speaks UTF-8 positions when the client offers
  them and UTF-16 otherwise, and asks the client to watch `**/*.sprig` when it
  can.

GitHub, which highlights with the same TextMate grammar as VS Code, will show
Sprig once Linguist knows the language. That is a later step.
