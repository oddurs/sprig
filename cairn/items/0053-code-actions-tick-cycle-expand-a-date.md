---
id: 53
uid: ac5f5c01-032b-43ce-ae87-cd8977320523
title: 'Code actions: tick, cycle, expand a date'
type: feature
status: done
milestone: v0.2
depends_on:
- 43
- 49
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: s
area: lsp
---

## Problem

Ticking an item in an editor should be one action, not a precise edit of the first character.

## Proposal

Code actions on an item line: mark done or reopen, cycle `-` `~` `x`, and expand a typed date to ISO. All go through the core's edit API.

## Acceptance criteria

- [x] Each action produces a text edit that touches only the mark or the date.
- [x] Ticking a recurring item rolls its due date instead of closing it.

## 2026-10-28

There are three code actions, all built on the core's edit API:

- **Tick** uses `edit::tick`, so a recurring item moves its due date.
- **Cycle** goes `-` to `~` to `x` and back to `-`. It never offers done for a recurring item.
- **Write a typed date out** replaces only the value, never the key.

The tests `ticking_touches_only_the_mark`, `ticking_a_recurring_item_rolls_its_due_date_instead_of_closing_it` and `a_typed_date_is_written_out_touching_only_its_value` assert the exact ranges. Document formatting uses `edit::format` too.

## Result

Code actions tick, cycle the mark and write out typed dates, each touching only what it means to; formatting comes free.
