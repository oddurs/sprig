---
id: 10
uid: 8dbdfbe8-21e5-4d47-8bdb-ad3084539086
title: Conformance runner and example format
type: feature
status: done
milestone: v0.1
depends_on:
- 8
- 9
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: suite
---

## Problem

The suite has to be runnable before it has examples, or examples get written in whatever format the first test happened to use.

## Proposal

Examples live in `tests/conformance/<section>/<NNN-slug>/`: one or more `.sprig` inputs (the entry is `main.sprig`), `expected.json` in the tree schema, and an optional `meta.toml` with `today = "YYYY-MM-DD"` and the spec rules the example cites. A Rust test harness discovers them without registration.

## Acceptance criteria

- [x] Adding an example directory needs no code change to run it.
- [x] A failure prints the example path, the rules it cites and a JSON diff of expected against actual.
- [x] A directory without `expected.json` fails the run instead of being skipped.
- [x] `tests/conformance/README.md` explains how to add an example in under ten lines.
- [x] `scripts/task test` runs the suite.

## 2026-10-28

`crates/sprig-core/tests/conformance.rs`. (1) Every directory `tests/conformance/<section>/<example>/` is discovered; nothing registers it. (2) A failure prints the example path, the rules from `meta.toml`, and one line per differing JSON path, with expected and actual. (3) A directory without `expected.json`, or without `main.sprig`, fails the run; both checked by planting one. (4) `tests/conformance/README.md` gives the steps for adding an example in seven lines. (5) It is a cargo integration test, so `scripts/task test` runs it. The runner also validates every expected and every actual tree against the schema, compares only the layers an example states, and never compares a diagnostic's wording. `SPRIG_BLESS=1` rewrites expected files for review.

## Result

A discover-by-directory runner that validates against the schema, reports per-path diffs with cited rules, and fails on missing expectations.
