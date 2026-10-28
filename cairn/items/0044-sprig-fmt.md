---
id: 44
uid: a8258ffb-9da9-4bd7-a31e-34bf96faf743
title: sprig fmt
type: feature
status: done
milestone: v0.2
depends_on:
- 16
- 43
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: cli
---

## Problem

Files written by hand, by agents and by paste drift in indentation and keep relative dates the format says to expand.

## Proposal

Normalise indentation to two spaces per level, expand typed dates to ISO, strip trailing whitespace, and end with one newline. Nothing else. `--check` lists files that would change and exits 1. `--today YYYY-MM-DD` for reproducible tests.

## Acceptance criteria

- [x] `fmt` is idempotent: a property test checks fmt(fmt(x)) == fmt(x) over the conformance inputs.
- [x] `fmt` never changes the syntax tree of a conformance input apart from indentation and expanded dates.
- [x] `sprig fmt --check` runs in this repository's CI over `examples/`.

## 2026-10-28

`sprig fmt [PATH...] [--check]`, built on `edit::format`. It indents two spaces per level, writes out typed dates on items, answers and titles (never notes), strips trailing whitespace and ends with one line ending. Line endings, a BOM, blank lines and comments otherwise stay. Comments keep their place relative to the lines above. An untitled file's first note keeps its indentation, since at the margin it would become the title. The rules are in `spec/tools.md` T2. `--check` follows ruff, gofmt and prettier: it writes nothing, prints one path per line, and exits 1. Files are replaced by write-then-rename. (1) `fmt_is_idempotent_and_keeps_the_tree_on_every_input` covers every conformance input and example, and a generated-plans proptest covers 3,000 cases in CI (50,000 run once). (2) The same tests compare syntax trees with spans dropped and date fields read against today: only layout and date spelling change. (3) `the_repositorys_own_examples_are_formatted` runs `sprig fmt --check examples` in every CI check job, on three platforms.

## Result

`sprig fmt` and `--check`: idempotent, tree-preserving, and holding the repository’s examples to it in CI.
