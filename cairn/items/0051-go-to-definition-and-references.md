---
id: 51
uid: 9348f150-ced4-40ef-abf5-842229ea2879
title: Go to definition and references
type: feature
status: planned
milestone: v0.2
depends_on:
- 49
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: lsp
---

## Problem

Following `after:[[lease^signed]]` by hand means opening a file and searching.

## Proposal

Definition on links and anchors, and references on an anchor: every `after:` and link that names it, across the workspace.

## Acceptance criteria

- [ ] Definition on `[[lease^signed]]` opens `lease.sprig` at that item.
- [ ] References on `^signed` list every line in the workspace that names it.
