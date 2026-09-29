---
id: 52
uid: 9abb4a1d-0d9b-45a7-9ea2-207d570b69a4
title: 'Hover: progress and what an item waits on'
type: feature
status: planned
milestone: v0.2
depends_on:
- 18
- 49
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: lsp
---

## Problem

The computed layer (progress, estimates, blockers) is invisible in a text editor.

## Proposal

Hovering an item shows done/total for its branch, estimated work left, effective owner, and each unfinished `after:` target.

## Acceptance criteria

- [ ] Hover over `+ [[kitchen]]` in `bakery.sprig` shows the kitchen file's totals.
- [ ] A blocked item's hover names every target it waits on.
