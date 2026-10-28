---
id: 50
uid: 17572da0-dc84-4cae-a9df-babca6094db5
title: Completion for people, anchors, files and fields
type: feature
status: done
milestone: v0.2
depends_on:
- 49
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: lsp
---

## Problem

Anchors and file names are where typos break links. Completion prevents them.

## Proposal

Complete `@` with people in the workspace, `^` and `after:^` with anchors in the file, `[[` with file names, `[[file^` with that file's anchors, and field keys (`due:`, `est:`, `every:`, `after:`).

## Acceptance criteria

- [x] Each trigger has a scripted test that asserts the offered items.
- [x] Completing `[[lease^` in `bakery.sprig` offers exactly the anchors in `lease.sprig`.

## 2026-10-28

Each trigger has a test in `crates/sprig-lsp/tests/lsp.rs`:

- `@` offers everyone in the workspace.
- `^` and `after:^` offer this file's anchors, but not the one being typed.
- `[[` offers other files, relative to this one.
- `[[lease^` offers exactly the anchors `lease.sprig` writes; the test computes them from the parser.
- Field keys are offered once their first letter is typed.

Nothing is offered in notes or comments (§2.6). The closing `]]` is added unless the editor already put it there.

## Result

Completion for people, anchors, file links, a file's anchors and the field keys that mean something, scripted per trigger.
