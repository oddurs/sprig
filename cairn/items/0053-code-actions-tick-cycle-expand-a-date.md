---
id: 53
uid: ac5f5c01-032b-43ce-ae87-cd8977320523
title: 'Code actions: tick, cycle, expand a date'
type: feature
status: planned
milestone: v0.2
depends_on:
- 43
- 49
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: s
area: lsp
---

## Problem

Ticking an item in an editor should be one action, not a precise edit of the first character.

## Proposal

Code actions on an item line: mark done or reopen, cycle `-` `~` `x`, and expand a typed date to ISO. All go through the core's edit API.

## Acceptance criteria

- [ ] Each action produces a text edit that touches only the mark or the date.
- [ ] Ticking a recurring item rolls its due date instead of closing it.
