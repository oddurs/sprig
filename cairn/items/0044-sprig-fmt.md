---
id: 44
uid: a8258ffb-9da9-4bd7-a31e-34bf96faf743
title: sprig fmt
type: feature
status: planned
milestone: v0.2
depends_on:
- 16
- 43
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: cli
---

## Problem

Files written by hand, by agents and by paste drift in indentation and keep relative dates the format says to expand.

## Proposal

Normalise indentation to two spaces per level, expand typed dates to ISO, strip trailing whitespace, and end with one newline. Nothing else. `--check` lists files that would change and exits 1. `--today YYYY-MM-DD` for reproducible tests.

## Acceptance criteria

- [ ] `fmt` is idempotent: a property test checks fmt(fmt(x)) == fmt(x) over the conformance inputs.
- [ ] `fmt` never changes the syntax tree of a conformance input apart from indentation and expanded dates.
- [ ] `sprig fmt --check` runs in this repository's CI over `examples/`.
