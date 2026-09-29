---
id: 79
uid: 5bbbea08-f1f5-4f48-a2a5-1e3acaf749c4
title: Core API documentation and public API review
type: docs
status: planned
milestone: v1.0
depends_on:
- 43
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: m
area: core
---

## Reader and question

A developer building their own Sprig tool on the core crate.

## Change

Complete docs.rs documentation with `#![deny(missing_docs)]`, compiling examples, and a review of the public API: `#[non_exhaustive]` where growth is expected, and nothing public that's only there for tests.

## Acceptance criteria

- [ ] `cargo doc` builds with missing_docs denied.
- [ ] Every public item has an example that compiles as a doctest or is explicitly exempted.
- [ ] The review's changes are listed in the CHANGELOG.
