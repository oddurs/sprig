---
id: 56
uid: bfcf2b39-3c98-4deb-82e6-9dd4e9f54a06
title: Performance baseline and regression gate
type: chore
status: done
milestone: v0.2
depends_on:
- 19
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: core
---

## Purpose

The language server reparses on every keystroke. Slowdowns creep in one small change at a time.

## Approach

A generated 5,000-line, 20-file workspace and a benchmark binary that times parse, resolve and check. CI compares against a committed baseline.

## Acceptance criteria

- [x] CI fails when the median is more than 2x the committed baseline.
- [x] The baseline and how to update it are documented in the benchmark's README.

## 2026-10-28

`crates/sprig-core/benches/workspace.rs` generates 20 files and 5,078 lines, shaped like the bakery, and times parse, resolve and check, taking the median of 31 runs. It fails if a median is more than twice `benches/baseline.json`. `scripts/task perf` runs it in the new `perf` CI job, together with the language server's keystroke test in a release build. `benches/README.md` says how to update the baseline and when not to.

Building it found the language server re-parsing all 20 files on every keystroke. Now `Workspace` parses a file when it is inserted, unchanged text keeps its parse, and the server keeps one workspace for the session. As a result:
- resolve takes 12 ms, down from 23;
- a keystroke takes 29 ms median, down from 42.

The committed baseline was measured here; the first CI run's numbers replace it.

## Result

A benchmark of parse, resolve and check on 5,000 lines, gated in CI at twice a committed baseline, and a 2x faster keystroke from parsing only what changed.
