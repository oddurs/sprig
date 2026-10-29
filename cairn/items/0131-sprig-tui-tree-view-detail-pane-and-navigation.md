---
id: 131
uid: 57ba274d-aafd-471c-9f55-cfc111d392ed
title: 'sprig tui: tree view, detail pane and navigation'
type: feature
status: done
milestone: v0.4
depends_on:
- 45
- 130
created: 2026-09-28
updated: 2026-10-29
closed_at: 2026-10-29
priority: p0
effort: l
area: cli
---

## Problem

The verbs answer one question each. Working through a plan means asking several in a row, and re-running commands to do it is slow.

## Proposal

`sprig tui [path]` opens the tree view of a file, with grafts expanded: `j`/`k` to move, `h`/`l` to fold, `g`/`G` to jump, and a detail pane with the item's `file:line`, status, owner, dates, estimate, progress, what it waits on, and its notes. A status line shows done/total, ready and waiting counts. `--screenshot WxH` renders one frame as text and exits, for snapshot tests and the docs. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [x] Snapshots of the tree view at 80x24 and 120x40 on `examples/bakery/` are committed and tested.
- [x] Every key in the help overlay has a test driving the pure core, with no terminal involved.
- [x] Below 80 columns the detail pane hides, and nothing wraps or overflows; a snapshot at 60x20 proves it.

## Result

sprig tui opens the tree with grafts mounted, a detail pane from 80 columns up, and a status line. --screenshot renders a frame as text: snapshots at 80x24, 120x40 and 60x20, where the pane hides and no line passes 60 columns. Every key in the help has a test driving the core.
