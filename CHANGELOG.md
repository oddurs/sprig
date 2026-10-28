# Changelog

All notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Spec draft 0.5:
  - It rules on every ambiguity Appendix B listed, folded into the numbered
    rules.
  - It states what the text left silent: quoted runs, escaped titles, links
    that must name a file, answers, indentation whitespace, date shortcuts,
    `est:` and `every:`, month-end recurrence, and due dates and priority
    that reach down.
  - New warnings: S0006 for an `every:` that is not a recurrence, S0206 for an
    `after:` naming something with nothing to do, and S0007 for a title naming
    a version of Sprig the tool does not read.
  - Dates have no time of day, an email address is not a person, and the
    title's `sprig:` key is reserved for a future major version.

- `sprig next`: open, unblocked work across a folder of plans, in progress
  first, then priority, then due date, with `--who` and `--limit`.
- `sprig check`: every problem as `file:line:col: severity[code]: message`,
  exit 1 on errors, and on warnings with `--strict`.
- `--today` on every command, so any answer can be reproduced.
- `sprig-core`: the parser and resolver. Grafts across files and directories,
  loops reported once, paths outside the workspace refused, settling,
  progress, estimates, people and blocking, with no I/O and no clock.
- `spec/diagnostics.md`, the catalogue of stable diagnostic codes.
- `spec/tree.schema.json` and spec Appendix D, the JSON tree format.
- `tests/conformance/`, the conformance suite: 55 examples for lines and
  indentation, 55 for tokens, fields and dates, and 59 for status, people,
  blocking and grafts, each section citing every rule it covers.
- Typed date shortcuts: `due:fri`, `start:+2w`, `oct3`, `q4` and the rest of
  §9.3 resolve against `--today`.
- The website runs `sprig-core` compiled to WebAssembly (109 KB compressed)
  instead of a JavaScript parser of its own.
- A command-line reference page on the site, generated from `sprig --help`,
  and the first-plan tutorial now ends in the terminal.
- `sprig fmt` and `sprig fmt --check`: two spaces per level, typed dates
  written out, trailing whitespace gone, and nothing else.
- `sprig tick REF...`: done, reopened, or a recurring item's due date moved
  on, changing only that byte range. Refs are `^anchor`, `file^anchor`,
  `file:line` or words from an item, and `-` reads them from standard input.
- `sprig_core::edit`, a byte-preserving edit API, and `spec/tools.md`, which
  states how refs, formatting and edits behave for other implementations.
- `sprig tree`, `sprig why` and `sprig status` (with `--oneline` for prompts).
- `--plain` on every reading command, tab-separated with a ref first, and
  `--color auto|always|never`, which respects `NO_COLOR`.
- `sprig lsp`, a language server built on `lsp-server`:
  - It republishes diagnostics for every file a change affects, so deleting an
    anchor flags the files that name it.
  - It completes people, anchors, file links and field keys.
  - It offers go to definition and references for anchors and links.
  - Hover shows progress, the work left, the owner and what an item waits on.
  - Code actions tick an item, cycle its mark and write out a typed date.
  - It formats documents.
- Editors:
  - a tree-sitter grammar in `editors/tree-sitter-sprig`, held to the core's
    reading of every conformance input;
  - TextMate grammar snapshots;
  - a VS Code extension that bundles the language server;
  - a Neovim setup tested in CI;
  - a guide to setting up each editor.
- A performance gate: parse, resolve and check on 5,000 lines must stay
  within twice a committed baseline. Re-inserting a file whose text is
  unchanged keeps its parse, so a keystroke reparses one file.
- `sprig completions bash|zsh|fish|powershell|elvish`, completing refs from
  the current folder, and `sprig man`.

### Changed

- A dropped item no longer holds up what waits on it: `after:` is satisfied by
  a done or a dropped target (spec §5.7). A later item still blocks.
- An item's text drops the brackets only of real links; `[[my notes]]` and
  `[[^a]]` stay as written.
