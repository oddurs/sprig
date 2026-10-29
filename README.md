# Sprig

[![ci](https://github.com/oddurs/sprig/actions/workflows/ci.yml/badge.svg)](https://github.com/oddurs/sprig/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Sprig is a plain-text format for plans, one item per line, where the first
character says what a line is and indentation says where it belongs. The
`sprig` command reads a folder of them and tells you what can be done now and
what is broken, for people and for the coding agents working beside them.

**Status:** before the first release. The format is
[draft 0.5](spec/sprig.md), held to it by 417 conformance examples, and its
design decisions are settled; a change from here goes through an
[RFC](rfcs/). The command, the language server, the TUI and the MCP server
all work today, tested on Linux, macOS and Windows. Nothing is published to crates.io or an editor marketplace yet, because the name is
still being cleared, so install from source for now.

## Why Sprig

Markdown checklists are everywhere, todo.txt is simple, and Org-mode can do
anything inside Emacs. Sprig is for the plan that outgrows them, and it stays
plain text:

- **It computes what is blocked.** `after:^van` makes an item wait, across
  files, and `sprig next` lists only what can start now.
- **A plan can span files.** `+ [[kitchen]]` grafts one plan into another, so
  each team keeps its own file and the roll-up stays live.
- **It is checked like code.** Every problem has a stable code, an exit status
  and a line number, for editors and for CI.
- **Agents can use it safely.** `sprig mcp` gives a coding agent the plan and
  checked, diffed edits, through the same parser every other tool uses.

[How Sprig compares](https://oddurs.github.io/sprig/compare/) sets it beside
Markdown task lists, todo.txt, TaskPaper, Org-mode and Taskwarrior, including
what each does better.

## Install

With Rust installed ([rustup.rs](https://rustup.rs)):

```sh
cargo install --locked --git https://github.com/oddurs/sprig sprig-cli
sprig --version
```

## A first plan

Save this as `plan.sprig`:

<!-- file: plan.sprig -->
```sprig
Move flats  @me

- Book a van  ^van  !!
~ Pack
  x Books
  - Kitchen  est:3h
- Move  after:^van  @me @jo
- Change address
  - Bank
  - Doctor
? Keep the old sofa?
```

Ask what can be done now:

```console
$ sprig next plan.sprig
- Book a van          Move flats        !!
- Kitchen             Pack
- Bank                Change address
- Doctor              Change address
? Keep the old sofa?  Move flats

5 ready · 1 waiting on something, not shown · 1 open question
```

*Move* is left out because it waits on the van (`after:^van`); it appears once
*Book a van* is ticked `x`. The most urgent work comes first: in progress, then
priority (`!` to `!!!`), then the nearest due date. `--who jo` shows only Jo's
work, and `--limit 3` the top three.

Then check it, as CI would:

```console
$ sprig check plan.sprig
```

Nothing printed, exit 0. Misspell the anchor on line 7 and check again:

<!-- edit: plan.sprig: after:^van => after:^vna -->
```console
$ sprig check plan.sprig
plan.sprig:7:9: error[S0201]: no item ^vna in plan.sprig
```

Every problem has a code from [`spec/diagnostics.md`](spec/diagnostics.md), and
the exit code is 1, so a CI job fails. In GitHub Actions it is one step:

```yaml
- run: cargo install --locked --git https://github.com/oddurs/sprig sprig-cli && sprig check --strict .
```

## The nine line starts

| Line | Means | Example |
| --- | --- | --- |
| `-` | to do | `- Book a van  due:2026-10-17` |
| `~` | doing | `~ Pack` |
| `x` | done | `x Books` |
| `/` | dropped, kept for the record | `/ Hire movers` |
| `>` | later, parked on purpose | `> Paint the new place` |
| `?` | a question | `? Keep the old sofa?` |
| `=` | the answer to the item above | `  = Yes, it fits the van.` |
| `#` | a group | `# Kitchen` |
| `+` | another file or branch grafted in | `+ [[kitchen]]` or `+ [[kitchen^oven]]` |

Anything else is a note on the item above it, and `//` starts a comment. On an
item, `@name` is who owns it, `^anchor` names it, `after:^anchor` makes it wait,
`due:2026-10-17` dates it, `est:3h` sizes it, and `#tag` tags it. Progress,
estimates and what is blocked are computed from the text and never written
into it.

## Going further

- [`examples/bakery/`](examples/bakery): six plans for opening a bakery that
  graft into one. Try `sprig next examples/bakery`.
- `sprig lsp`: a language server for any editor that speaks LSP. It shows
  problems as you type, including ones another file causes, and completes
  people, anchors and file links. It also follows links, and its hover shows
  progress and what an item waits on. Point your editor's LSP client at
  `sprig lsp` for `*.sprig` files.
- [Sprig in the shell](https://oddurs.github.io/sprig/docs/shell/): `find`,
  `agenda`, `add`, `mark`, `graph`, `export` and `import`, and recipes for fzf,
  jq, a pre-commit hook and a calendar feed. [`action/`](action) checks plans
  in GitHub Actions and summarises what a pull request did to them.
- [`sprig tui`](https://oddurs.github.io/sprig/docs/tui/): the tree, what's
  next and the agenda on one screen, with vim keys, and reloads when an editor
  or an agent saves.
- [Plans for coding agents](https://oddurs.github.io/sprig/docs/agents/): the
  `PLAN.sprig` convention, `sprig agent --write AGENTS.md`, and `sprig mcp`,
  which lets an agent read the plans and change them through checked,
  diffed edits.
- [`docs/stability.md`](docs/stability.md): what will not change under you
  after 1.0, each promise with the test that keeps it, and
  [`rfcs/`](rfcs), how the spec changes.
- [`spec/sprig.md`](spec/sprig.md): the specification, every rule numbered,
  each with an example, and [`tests/conformance/`](tests/conformance) holding
  the implementation to it.
- [The website](https://oddurs.github.io/sprig/), with a
  [playground](https://oddurs.github.io/sprig/play/) and a
  [first-plan tutorial](https://oddurs.github.io/sprig/docs/first-plan/).
- [`ROADMAP.md`](ROADMAP.md): what comes next, generated from the
  [cairn](https://oddurs.github.io/cairn) items in [`cairn/items/`](cairn/items).

## Development

```sh
scripts/setup                            # once per clone: hooks and toolchain
scripts/agent doctor                     # is this checkout ready?
scripts/agent start feat/0017-grafts     # a worktree and branch of its own
scripts/task check                       # format, lint, test, build, backlog
scripts/task site                        # build the website and check its links
scripts/agent pr                         # check, push, open the pull request
```

`main` only advances through a merged pull request. [CONTRIBUTING.md](CONTRIBUTING.md)
has the whole loop, and [ARCHITECTURE.md](ARCHITECTURE.md) is the map of the
code: what each crate does, how data flows through it, and the invariants that
hold it together. Questions go where [SUPPORT.md](SUPPORT.md) says.

## License

Code is [MIT](LICENSE). The specification in `spec/` is dedicated to the public
domain under [CC0](spec/LICENSE), so anyone can implement it without asking.
