---
id: 23
uid: 6b6345f9-5321-42e5-93fa-25441fb5eddb
title: 'sprig next: what can be done now'
type: feature
status: done
milestone: v0.1
depends_on:
- 19
- 22
created: 2026-09-28
updated: 2026-10-28
closed_at: 2026-10-28
priority: p0
effort: m
area: cli
---

## Problem

The first question anyone asks a plan is what to do next. The playground's Next up view answers it; the terminal needs the same answer.

## Proposal

List open, unblocked leaves ranked by in-progress first, then priority, then inherited due date, then source order, matching the playground in `design/sprig-draft-0.2.html`. Each line shows its path through the tree. Options: `--who NAME` and `--limit N`. Add the six playground files, with ISO dates, as `examples/bakery/`.

## Acceptance criteria

- [x] For `examples/bakery/` the output matches a committed snapshot, and its order matches the playground's Next up for the same files and date.
- [x] `--who teo` shows only items whose effective owner is teo.
- [x] Items waiting on something are excluded, and the count of excluded items is printed.

## 2026-10-28

`sprig_core::next` ranks; the CLI prints one line per item: mark and text, its path through the tree (clipped from the left, so the nearest parent survives), due date with days left, and priority. Then one summary line. (1) Snapshots of `sprig next --today 2026-10-01` for `examples/bakery/bakery.sprig` and for the whole folder are in `crates/sprig-cli/tests/snapshots/`. The bakery.sprig order equals the playground's Next up (the reference parser's `walkLeaves` and sort, run under node), item for item, on four dates. For a folder, the plans no file grafts whole are read in path order, and an item mounted twice is listed once. (2) `--who teo`, or `--who @teo`, lists the three items teo owns, each also in the unfiltered list. Ownership is judged per tree, since a graft can reassign a branch (§6.1). (3) Waiting items are left out and counted: "20 ready · 12 waiting on something, not shown · 2 open questions". `--limit N` is tested, and nothing ready exits 1. The six playground files with ISO dates were already in `examples/bakery/`.

## Result

`sprig next` matches the playground on the bakery, with `--who` and `--limit`, and counts what it leaves out.
