---
id: 50
uid: 17572da0-dc84-4cae-a9df-babca6094db5
title: Completion for people, anchors, files and fields
type: feature
status: planned
milestone: v0.2
depends_on:
- 49
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: lsp
---

## Problem

Anchors and file names are where typos break links. Completion prevents them.

## Proposal

Complete `@` with people in the workspace, `^` and `after:^` with anchors in the file, `[[` with file names, `[[file^` with that file's anchors, and field keys (`due:`, `est:`, `every:`, `after:`).

## Acceptance criteria

- [ ] Each trigger has a scripted test that asserts the offered items.
- [ ] Completing `[[lease^` in `bakery.sprig` offers exactly the anchors in `lease.sprig`.
