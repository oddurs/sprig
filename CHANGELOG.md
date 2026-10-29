# Changelog

All notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- The conformance suite holds 417 examples, with every numbered rule of the
  spec cited:
  - Dates, shortcuts and estimates are checked against an oracle written
    from the spec, not from the parser.
  - Edit examples hold ticking (§10.4) and formatting (§3.3, §9.4) to exact
    bytes.
  - A coverage report in `tests/conformance/README.md` lists each rule's
    examples, and CI fails when a rule has none.
- `docs/stability.md`, the stability policy. Every promise names the test
  that keeps it, and a test fails if that test disappears.
- `--plain` snapshots for `next`, `check`, `tree`, `status`, `find` and
  `agenda`.
- `rfcs/`, the process for changing the spec, with its template. The rulings
  on times of day, email addresses and file versions are RFCs 0001 to 0003.
  CONTRIBUTING says who decides and how long review stays open.
- Fuzz targets for the parser and the edit API (`scripts/task fuzz`), seeded
  from the suite, and a minute of each in CI.
- The site's reference gains the MCP tools and the diagnostics, generated
  from the server and the catalogue. CI fails on a command, option or
  diagnostic without documentation.
- The tutorial ends with ticking the plan and handing it to an agent.

### Changed

- `sprig-core`'s public API, reviewed for 1.0:
  - Every public item is documented, and every public function has an
    example that runs as a test.
  - These are now `#[non_exhaustive]`, so a later version can add to them:
    `Code`, `Wait`, `Piece`, `query::Is`, `QueryError`, `RefError` and
    `EditError`.
  - These are no longer public, as nothing outside the crate used them:
    `syntax::estimate_hours`, `date::recurs`, `Date::add_month` and
    `Tokens::graft_link`.
- §10.3 says what a count in `every:` counts: days or weeks.

### Fixed

- The tree-sitter grammar reads a file whose last line has no line ending,
  which the new conformance examples found. An external scanner ends a line
  at a newline or at the end of the file.
- The GitHub Action's summary named an unblocked item by a wrong path
  (`plans/bakery/bakery/bakery.sprig`) when it checked a folder of folders or
  a single file. Only a folder one level deep came out right.

- `sprig tui`: the tree, next up and the agenda on one screen.
  - Vim keys, a detail pane, and a filter that keeps every match's ancestors.
  - Ticks, marks, adds and undo through the edit API.
  - Live reload when a file changes, keeping the cursor on its item.
  - A write is refused if its line changed on disk since the TUI showed it,
    so an editor's or an agent's save is never overwritten.
  - `--screenshot WxH` prints a frame as text, and `--keys` presses keys
    first.
- `spec/tools.md` T7, and a guide to the TUI.
- `sprig mcp`: the plans for coding agents, over the Model Context Protocol.
  - Five read tools return what the matching `--json` command prints.
  - Five write tools go through the edit API, refuse a change that would add
    a `check` error, and return the diff they applied and what stopped
    waiting.
  - `--max-new-items`, 20 by default, caps how many items a session may add.
  - `--config` prints the snippet that registers the server with a client.
- `sprig agent` prints the convention for AGENTS.md or CLAUDE.md, and
  `--write FILE` keeps it in a marked block there.
- `status --json` and `check --json`.
- `spec/tools.md` T6, and a guide to plans for coding agents with a worked
  session.
- Tools for the shell, each sharing the core's edit API or tree:
  - `sprig find` searches by words, `@who`, `#tag`, `is:STATE` and
    `due:<DATE`.
  - `sprig agenda` shows dated work by day.
  - `sprig mark`, `sprig answer` and `sprig add` change a plan from the
    command line and say what stopped waiting.
  - `sprig edit` opens an editor at an item.
  - `sprig graph` prints the dependencies for Graphviz or Mermaid.
  - `sprig export` writes JSON, iCalendar or CSV.
  - `sprig import` reads Markdown checklists, todo.txt and GitHub issues.
  - `next --json` and `find --json` print items in the tree format.
- `spec/tools.md` T5: what queries mean, the agenda, calendar UIDs, graph ids
  and conversions.
- A GitHub Action in `action/`. It checks plans, annotates problems, and
  summarises a pull request's effect on done counts and on what it unblocked.
- A guide to Sprig in the shell, whose recipes run in CI.
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
