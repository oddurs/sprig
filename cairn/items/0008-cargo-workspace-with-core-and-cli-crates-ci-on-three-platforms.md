---
id: 8
uid: 6f82c564-e4f4-4bc0-a564-754ec34bff25
title: Cargo workspace with core and CLI crates, CI on three platforms
type: chore
status: done
milestone: v0.1
created: 2026-09-28
updated: 2026-09-28
closed_at: 2026-09-28
priority: p0
effort: s
area: release
---

## Purpose

Every later item needs somewhere to land and a CI that runs on the platforms v1.0 promises.

## Approach

A Cargo workspace with `crates/sprig-core` (library, no I/O, no clock) and `crates/sprig-cli` (the `sprig` binary). Wire `scripts/task` so `fmt`, `lint`, `test`, `build` and `check` call cargo. CI runs `scripts/task check` on ubuntu, macos and windows. Code is MIT, the spec directory is CC0. Crate names follow the answer to the name spike; use working names until then.

## Acceptance criteria

- [x] `scripts/task check` passes locally and in CI on all three platforms.
- [x] Clippy runs with warnings denied and rustfmt runs in check mode.
- [x] `sprig-core` has no filesystem or clock access; a CI step greps for `std::fs` and `SystemTime` in it and fails if found.
- [x] LICENSE (MIT) at the root and spec/LICENSE (CC0) exist.

## 2026-09-28

Delivered by the repository bootstrap (initial commit 4ec974d) and hardened in #1 and #3. Evidence per criterion: (1) scripts/task check green locally and in CI run 36502243610 on ubuntu, macos and windows; (2) scripts/task lint runs clippy --all-targets -D warnings and fmt:check runs cargo fmt --check; (3) scripts/task lint greps crates/sprig-core/src for std::(fs|env|net|process), SystemTime and Instant::now, runs in every CI check job, and was shown to fail with exit 1 when a SystemTime::now call was added; (4) LICENSE is MIT (2026, Oddur Sigurdsson) and spec/LICENSE is the CC0 1.0 legal code.

## Result

Cargo workspace (sprig-core, sprig-cli), scripts/task seam, hooks, and CI on Linux, macOS and Windows plus a backlog job, all green. Crate names remain working names and publish = false until the name spike (0006) answers.
