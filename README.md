# Sprig

[![ci](https://github.com/oddurs/sprig/actions/workflows/ci.yml/badge.svg)](https://github.com/oddurs/sprig/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A plain-text format for plans. A shopping list and an eighteen-month build use
the same handful of marks, nest as deep as they need to, and link across files,
so small plans add up to a big one. People and coding agents read, edit and
check the same file.

```
Open the bakery  @dana  target:2026-12-05

# Money
  Budget is 180k. Over 10k needs two quotes.
  x Business plan
  ~ Equipment loan  due:2026-10-03  !!
  - Supplier credit  @teo  after:[[lease^signed]]
+ [[kitchen]]  @teo
? Kiln & Crumb, or Mill Street Bread?
  = Kiln & Crumb. Trademark search is clear.
```

`-` to do, `~` doing, `x` done, `/` dropped, `>` later, `?` a question, `=` its
answer, `#` a group, `+` another file grafted in as a branch. Indentation is
structure. Progress, estimates and what's blocked are computed, never written.

## Status

Sprig is a draft (0.2) with a plan. There is no parser yet, and nothing to
install. What exists today:

- [`design/sprig-draft-0.2.html`](design/sprig-draft-0.2.html): the language,
  the settled design arguments, and a working playground with five linked
  sample files. Open it in a browser; it runs offline apart from web fonts.
- [`design/ecosystem-plan.html`](design/ecosystem-plan.html): how Sprig gets
  from a draft to something people use, with the roadmap written in Sprig.
- [`ROADMAP.md`](ROADMAP.md): the backlog from v0.1 to 1.0, generated from the
  [cairn](https://oddurs.github.io/cairn) items in [`cairn/items/`](cairn/items).
- A Rust workspace (`crates/sprig-core`, `crates/sprig-cli`) with a `sprig`
  binary that so far only answers `--help` and `--version`.

## Quickstart

Try the language in the playground:

```sh
open design/sprig-draft-0.2.html      # xdg-open on Linux, start on Windows
```

Build the CLI from source (Rust 1.98, installed automatically by rustup from
`rust-toolchain.toml`):

```sh
cargo install --path crates/sprig-cli
sprig --version
```

See what's ready to work on, with [cairn](https://oddurs.github.io/cairn)
installed:

```sh
cairn next
```

## Development

```sh
scripts/setup                            # once per clone: hooks and toolchain
scripts/agent doctor                     # is this checkout ready?
scripts/agent start feat/0017-grafts     # a worktree and branch of its own
scripts/task check                       # format, lint, test, build, backlog
scripts/agent pr                         # check, push, open the pull request
```

`main` only advances through a merged pull request. [CONTRIBUTING.md](CONTRIBUTING.md)
has the whole loop.

## License

Code is [MIT](LICENSE). The specification, which `spec/` will hold from item
0007 on, is dedicated to the public domain under [CC0](spec/LICENSE), so anyone
can implement it without asking.
