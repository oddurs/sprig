---
id: 52
uid: 9abb4a1d-0d9b-45a7-9ea2-207d570b69a4
title: 'Hover: progress and what an item waits on'
type: feature
status: done
milestone: v0.2
depends_on:
- 18
- 49
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: lsp
---

## Problem

The computed layer (progress, estimates, blockers) is invisible in a text editor.

## Proposal

Hovering an item shows done/total for its branch, estimated work left, effective owner, and each unfinished `after:` target.

## Acceptance criteria

- [x] Hover over `+ [[kitchen]]` in `bakery.sprig` shows the kitchen file's totals.
- [x] A blocked item's hover names every target it waits on.

## 2026-10-28

Hover on an item or the title shows:

- status;
- done of total, and how many are doing;
- work left;
- people;
- due and start dates, relative to today;
- each unfinished `after:` target with its file and line, or the start date it waits for;
- ready to close.

On a graft it also names what it mounts. The tests are `hovering_a_graft_shows_the_grafted_files_totals`, which checks against the core's own totals for `kitchen.sprig`, and `hovering_a_blocked_item_names_everything_it_waits_on`.

## Result

Hover shows the computed layer: progress, estimate, owner, dates and every blocker.
