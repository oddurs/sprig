# Changelog

All notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
- `sprig completions bash|zsh|fish|powershell|elvish`, completing refs from
  the current folder, and `sprig man`.
