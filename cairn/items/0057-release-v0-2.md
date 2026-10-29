---
id: 57
uid: 6de9b81f-401b-4cda-ab95-536280418539
title: Release v0.2
type: chore
status: planned
milestone: v0.2
depends_on:
- 44
- 47
- 54
created: 2026-09-28
updated: 2026-10-29
priority: p0
effort: s
area: release
---

## Purpose

Ship editor support and the formatter.

## Approach

CHANGELOG entry, `v0.2.0` tag, crates published, VS Code extension published, release notes.

## Acceptance criteria

- [ ] The v0.2 milestone's Done-when list is fully ticked.
- [ ] A fresh VS Code install with only the extension shows diagnostics on `examples/bakery/`.

## 2026-10-29

Waits on the name (0006): the crate cannot be published, and a GitHub release would fix a name that may change. Everything a release needs from the code is in place: the install job runs the README's command on all three platforms, and CHANGELOG.md has the entries.
