---
id: 45
uid: 178d65e6-1f89-4506-98d7-d3b4acb5824f
title: sprig tree
type: feature
status: done
milestone: v0.2
depends_on:
- 18
- 22
- 119
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p1
effort: s
area: cli
---

## Problem

There is no way to see a plan's shape and progress in the terminal, which the playground's Tree view shows at a glance.

## Proposal

Print a file as an indented tree with a status glyph, text, people and dates, and for each parent `done/total` plus the estimate left. `--depth N` and `--hide-done`.

## Acceptance criteria

- [x] Output for `examples/bakery/bakery.sprig` matches a committed snapshot, with grafted branches shown under their graft line.
- [x] Colour follows `--color auto|always|never` and `NO_COLOR`.

## 2026-10-28

`sprig tree [PATH...] [--depth N] [--hide-done]`. One row per item: a tree prefix; the mark, or the status an ancestor settled it to; the text; then the graft target, people, priority, due date, and what it waits on; then done/total and the estimate left for parents. (1) `tree_for_the_bakery_matches_its_snapshot_with_grafts_under_their_lines` (`tests/snapshots/tree-bakery.txt`): lease, kitchen and launch appear under their `+` lines, groceries under Stock the shelves. (2) Colour goes through the output layer, so `--color` and `NO_COLOR` apply, tested with `next` and `status`. `--plain` gives ref, depth, mark, text, status, done and total.

## Result

`sprig tree` shows a plan with grafts expanded, progress and waits, under the shared output rules.
