---
id: 17
uid: 42441099-b1f2-4c91-96f0-84d7979762d0
title: Load a workspace and resolve grafts
type: feature
status: planned
milestone: v0.1
depends_on:
- 12
- 15
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: core
---

## Problem

`+ [[kitchen]]` and `+ [[kitchen^oven]]` are what make many small plans one big one. Resolution has to be correct, and safe on hostile paths.

## Proposal

The CLI hands the core a set of files by path. The core resolves grafts relative to the grafting file, mounts whole files or single branches, and detects loops and missing targets.

## Acceptance criteria

- [ ] A graft to a missing file or anchor produces a diagnostic on the graft line.
- [ ] A three-file loop produces one error on the closing graft line and no stack overflow.
- [ ] `[[file^id]]` mounts exactly that branch and its notes.
- [ ] A graft path that leaves the workspace root (`[[../../etc/passwd]]`) is refused with a diagnostic, covered by a test.
