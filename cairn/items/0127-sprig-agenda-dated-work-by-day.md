---
id: 127
uid: ed014791-1923-4506-a104-762480e0bc8f
title: 'sprig agenda: dated work by day'
type: feature
status: planned
milestone: v0.3
depends_on:
- 16
- 19
- 119
created: 2026-09-28
updated: 2026-09-28
priority: p2
effort: s
area: cli
---

## Problem

Due dates are spread across files. Nobody can see the next two weeks without exporting to a calendar.

## Proposal

`sprig agenda [--days N]` lists open dated items grouped by day across the folder, with late items first and recurring items shown with their rule. Its date walk is shared with `export --format ics` (0064). The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] Output for `examples/bakery/` with `--today 2026-10-01` matches a committed snapshot.
- [ ] A recurring item appears on its next due date only, not repeated through the window.
- [ ] The agenda and the ICS export list the same items for the same window; a test compares them.
