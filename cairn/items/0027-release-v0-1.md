---
id: 27
uid: 61474118-4168-4e40-b091-4c654b2c4061
title: Release v0.1
type: chore
status: planned
milestone: v0.1
depends_on:
- 6
- 11
- 23
- 25
- 26
created: 2026-09-28
updated: 2026-10-28
priority: p0
effort: s
area: release
---

## Purpose

Put v0.1 where a stranger can install it.

## Approach

CHANGELOG entry, `v0.1.0` tag, publish the crates under the cleared name, GitHub release with notes.

## Acceptance criteria

- [ ] `cargo install` of the published crate succeeds in a clean CI job on macOS, Linux and Windows and runs `sprig --version`.
- [ ] The GitHub release links the spec and the README.
- [ ] The v0.1 milestone's Done-when list is fully ticked.

## 2026-10-28

Prepared, not released. The CHANGELOG has an Unreleased entry for v0.1. The CLI's dependency on `sprig-core` carries a version, so the crates can be published. `scripts/task install` and a new `install` CI job run `cargo install --locked --path crates/sprig-cli` on Linux, macOS and Windows, then `sprig --version`. Not done, and not mine to do without the owner: tagging `v0.1.0`, publishing the crates (blocked on the name, 0006, and on `publish = false`), and the GitHub release. When the name is settled, switch the install job to the published crate.
