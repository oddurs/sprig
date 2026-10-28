---
id: 19
uid: b8947f0b-959a-446c-94a3-cb0cabdab9e1
title: Compute blocked items
type: feature
status: done
milestone: v0.1
depends_on:
- 18
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: s
area: core
---

## Problem

Blocked is computed, never written. It is what `sprig next` needs to exclude work that can't start.

## Proposal

An open item is blocked while any `after:` target it has, or an ancestor has, is unfinished. Targets are `^id` in the same file, `[[file^id]]` and `[[file]]` (the whole file done).

## Acceptance criteria

- [x] Local, cross-file and whole-file targets each have a unit test.
- [x] Blocking inherited from an ancestor appears on every open descendant.
- [x] An unknown target produces a diagnostic, and the item counts as blocked.

## 2026-10-28

`block()` in `workspace.rs`. A reference is answered in the target file's own tree, so the answer is the same whoever asks. (1) Local `^id`, cross-file `[[b^signed]]` and whole-file `[[b]]` targets are unit tested, including a done target that does not block. (2) Blocking inherited from an ancestor reaches every open descendant and no settled one, and inherited blocking is not listed as the item's own wait. (3) Unknown targets get S0201, S0202 or S0203, an `after:` that is not a reference gets S0205, and the item counts as blocked. Future `start:` dates block too (§7.5, draft 0.4). On the bakery, the waiting list equals the reference parser's for every file and the four dates above.

## Result

Blocked is computed from references, start dates and ancestors, never written, and matches the playground.
