---
id: 23
uid: 6b6345f9-5321-42e5-93fa-25441fb5eddb
title: 'sprig next: what can be done now'
type: feature
status: planned
milestone: v0.1
depends_on:
- 19
- 22
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: cli
---

## Problem

The first question anyone asks a plan is what to do next. The playground's Next up view answers it; the terminal needs the same answer.

## Proposal

List open, unblocked leaves ranked by in-progress first, then priority, then inherited due date, then source order, matching the playground in `design/sprig-draft-0.2.html`. Each line shows its path through the tree. Options: `--who NAME` and `--limit N`. Add the six playground files, with ISO dates, as `examples/bakery/`.

## Acceptance criteria

- [ ] For `examples/bakery/` the output matches a committed snapshot, and its order matches the playground's Next up for the same files and date.
- [ ] `--who teo` shows only items whose effective owner is teo.
- [ ] Items waiting on something are excluded, and the count of excluded items is printed.
