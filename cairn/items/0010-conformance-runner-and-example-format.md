---
id: 10
uid: 8dbdfbe8-21e5-4d47-8bdb-ad3084539086
title: Conformance runner and example format
type: feature
status: planned
milestone: v0.1
depends_on:
- 8
- 9
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: suite
---

## Problem

The suite has to be runnable before it has examples, or examples get written in whatever format the first test happened to use.

## Proposal

Examples live in `tests/conformance/<section>/<NNN-slug>/`: one or more `.sprig` inputs (the entry is `main.sprig`), `expected.json` in the tree schema, and an optional `meta.toml` with `today = "YYYY-MM-DD"` and the spec rules the example cites. A Rust test harness discovers them without registration.

## Acceptance criteria

- [ ] Adding an example directory needs no code change to run it.
- [ ] A failure prints the example path, the rules it cites and a JSON diff of expected against actual.
- [ ] A directory without `expected.json` fails the run instead of being skipped.
- [ ] `tests/conformance/README.md` explains how to add an example in under ten lines.
- [ ] `scripts/task test` runs the suite.
