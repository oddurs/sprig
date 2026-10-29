---
id: 79
uid: 5bbbea08-f1f5-4f48-a2a5-1e3acaf749c4
title: Core API documentation and public API review
type: docs
status: done
milestone: v1.0
depends_on:
- 43
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p1
effort: m
area: core
---

## Reader and question

A developer building their own Sprig tool on the core crate.

## Change

Complete docs.rs documentation with `#![deny(missing_docs)]`, compiling examples, and a review of the public API: `#[non_exhaustive]` where growth is expected, and nothing public that's only there for tests.

## Acceptance criteria

- [x] `cargo doc` builds with missing_docs denied.
- [x] Every public item has an example that compiles as a doctest or is explicitly exempted.
- [x] The review's changes are listed in the CHANGELOG.

## Result

sprig-core builds with missing_docs denied, and cargo doc runs with warnings denied in lint. Every module has an example that runs as a doctest (now part of scripts/task test), and tests/api.rs holds every public function to one. The review made seven enums non_exhaustive and four helpers crate-private, as the CHANGELOG lists.
