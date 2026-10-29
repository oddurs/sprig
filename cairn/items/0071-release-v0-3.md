---
id: 71
uid: cb1c88bc-25eb-4dd7-984c-bc44c63af6c2
title: Release v0.3
type: chore
status: planned
milestone: v0.3
depends_on:
- 57
- 60
- 62
created: 2026-09-28
updated: 2026-10-29
priority: p0
effort: s
area: release
---

## Purpose

Ship agent access, export and import.

## Approach

CHANGELOG entry, `v0.3.0` tag, crates and extension published, release notes with an MCP client configuration example.

## Acceptance criteria

- [ ] The v0.3 milestone's Done-when list is fully ticked.

## 2026-10-29

Waits on the name (0006): the crate cannot be published, and a GitHub release would fix a name that may change. Everything a release needs from the code is in place: the install job runs the README's command on all three platforms, and CHANGELOG.md has the entries.
