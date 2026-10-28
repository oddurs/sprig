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
- `tests/conformance/`, the conformance suite, with 55 examples for lines and
  indentation.
- Typed date shortcuts: `due:fri`, `start:+2w`, `oct3`, `q4` and the rest of
  §9.3 resolve against `--today`.
- The website runs `sprig-core` compiled to WebAssembly (109 KB compressed)
  instead of a JavaScript parser of its own.
- A command-line reference page on the site, generated from `sprig --help`,
  and the first-plan tutorial now ends in the terminal.
