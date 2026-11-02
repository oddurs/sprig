---
title: Set up your editor
description: Highlighting, problems as you type, completion and ticking for .chea files in VS Code, Neovim, Helix, Zed and any editor that speaks LSP.
---

Every editor gets the same two things from Chea:

- **The language server**, `chea lsp`, which comes with the `chea` command.
  It shows problems as you type, including the ones a change in one file
  causes in another. It also completes `@people`, `^anchors`, `[[file]]` links
  and field keys, goes to the definition of an anchor or link, finds every
  line that names it, shows progress and blockers on hover, and ticks items
  and formats files.
- **A grammar** for highlighting: the TextMate grammar in
  `editors/chea.tmLanguage.json`, or the tree-sitter grammar in
  `editors/tree-sitter-chea`.

Install `chea` first; the editors below find it on your `PATH`:

```sh
cargo install --locked --git https://github.com/oddurs/chea chea-cli
chea --version
```

The language server treats the folder your editor opened as the workspace, so
open the folder that holds your plans rather than a single file.

## VS Code

The extension in `editors/vscode` registers `.chea` files, highlights them
and runs the language server it bundles, so it needs nothing else installed.
It isn't on the Marketplace yet. Until it is, install a package built by CI:

1. Download `chea-<platform>.vsix` from the latest run of the
   [vscode workflow](https://github.com/oddurs/chea/actions/workflows/vscode.yml).
2. Run **Extensions: Install from VSIX…** from the command palette and pick it.

To use a different `chea`, set `chea.server.path`.

CI holds this to a test that opens `examples/bakery/` in VS Code with only the
extension installed and checks the problems it reports.

## Neovim

Neovim 0.9 or later needs no plugins. Copy
[`editors/neovim/chea.lua`](https://github.com/oddurs/chea/blob/main/editors/neovim/chea.lua)
into your config, for example as `~/.config/nvim/plugin/chea.lua`. It sets the
filetype, starts `chea lsp` for every `.chea` buffer, and turns on tree-sitter
highlighting when the parser is installed.

For highlighting, build the parser and copy the queries:

```sh
git clone https://github.com/oddurs/chea
cd chea/editors/tree-sitter-chea
tree-sitter build -o ~/.config/nvim/parser/chea.so
mkdir -p ~/.config/nvim/queries/chea
cp queries/highlights.scm ~/.config/nvim/queries/chea/
```

If you use nvim-lspconfig, the same server can be added as a custom
configuration with `cmd = { 'chea', 'lsp' }` and `filetypes = { 'chea' }`.

CI follows this section on a clean config: it loads `chea.lua` and the
parser, opens `examples/bakery/lease.chea` and deletes `^signed`. It then
waits for `bakery.chea` and `kitchen.chea`, which name it, to report a
problem.

## Helix

Add to `~/.config/helix/languages.toml`, with `rev` set to a commit of this
repository:

```toml
[language-server.chea]
command = "chea"
args = ["lsp"]

[[language]]
name = "chea"
scope = "source.chea"
file-types = ["chea"]
roots = [".git"]
comment-token = "//"
indent = { tab-width = 2, unit = "  " }
language-servers = ["chea"]

[[grammar]]
name = "chea"
source = { git = "https://github.com/oddurs/chea", rev = "COMMIT", subpath = "editors/tree-sitter-chea" }
```

Then fetch and build the grammar, and copy the highlight queries into your
runtime directory:

```sh
hx --grammar fetch && hx --grammar build
mkdir -p ~/.config/helix/runtime/queries/chea
cp editors/tree-sitter-chea/queries/highlights.scm ~/.config/helix/runtime/queries/chea/
```

`hx --health chea` should then list the language server and highlighting.

## Zed

Zed reaches a language server and a grammar only through an extension, and
there isn't one for Chea yet (cairn 0147). Until there is, Zed has no Chea
support.

## Other editors

Any editor with an LSP client can run `chea lsp` over standard input and
output for files ending in `.chea`:

- **Emacs:** with eglot, add `(add-to-list 'eglot-server-programs '(chea-mode . ("chea" "lsp")))`.
- **Sublime Text:** with the LSP package, add a client whose command is
  `["chea", "lsp"]` and whose selector matches `.chea` files.
- **Anything else:** the server speaks UTF-8 positions when the client offers
  them and UTF-16 otherwise, and asks the client to watch `**/*.chea` when it
  can.

GitHub, which highlights with the same TextMate grammar as VS Code, will show
Chea once Linguist knows the language. That is a later step.
