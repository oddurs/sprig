---
id: 8
uid: 6f82c564-e4f4-4bc0-a564-754ec34bff25
title: Cargo workspace with core and CLI crates, CI on three platforms
type: chore
status: planned
milestone: v0.1
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: release
---

## Purpose

Every later item needs somewhere to land and a CI that runs on the platforms v1.0 promises.

## Approach

A Cargo workspace with `crates/sprig-core` (library, no I/O, no clock) and `crates/sprig-cli` (the `sprig` binary). Wire `scripts/task` so `fmt`, `lint`, `test`, `build` and `check` call cargo. CI runs `scripts/task check` on ubuntu, macos and windows. Code is MIT, the spec directory is CC0. Crate names follow the answer to the name spike; use working names until then.

## Acceptance criteria

- [ ] `scripts/task check` passes locally and in CI on all three platforms.
- [ ] Clippy runs with warnings denied and rustfmt runs in check mode.
- [ ] `sprig-core` has no filesystem or clock access; a CI step greps for `std::fs` and `SystemTime` in it and fails if found.
- [ ] LICENSE (MIT) at the root and spec/LICENSE (CC0) exist.
