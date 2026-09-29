---
id: 19
uid: b8947f0b-959a-446c-94a3-cb0cabdab9e1
title: Compute blocked items
type: feature
status: planned
milestone: v0.1
depends_on:
- 18
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: core
---

## Problem

Blocked is computed, never written. It is what `sprig next` needs to exclude work that can't start.

## Proposal

An open item is blocked while any `after:` target it has, or an ancestor has, is unfinished. Targets are `^id` in the same file, `[[file^id]]` and `[[file]]` (the whole file done).

## Acceptance criteria

- [ ] Local, cross-file and whole-file targets each have a unit test.
- [ ] Blocking inherited from an ancestor appears on every open descendant.
- [ ] An unknown target produces a diagnostic, and the item counts as blocked.
