---
id: 131
uid: 57ba274d-aafd-471c-9f55-cfc111d392ed
title: 'sprig tui: tree view, detail pane and navigation'
type: feature
status: planned
milestone: v0.4
depends_on:
- 45
- 130
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: l
area: cli
---

## Problem

The verbs answer one question each. Working through a plan means asking several in a row, and re-running commands to do it is slow.

## Proposal

`sprig tui [path]` opens the tree view of a file, with grafts expanded: `j`/`k` to move, `h`/`l` to fold, `g`/`G` to jump, and a detail pane with the item's `file:line`, status, owner, dates, estimate, progress, what it waits on, and its notes. A status line shows done/total, ready and waiting counts. `--screenshot WxH` renders one frame as text and exits, for snapshot tests and the docs. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] Snapshots of the tree view at 80x24 and 120x40 on `examples/bakery/` are committed and tested.
- [ ] Every key in the help overlay has a test driving the pure core, with no terminal involved.
- [ ] Below 80 columns the detail pane hides, and nothing wraps or overflows; a snapshot at 60x20 proves it.
