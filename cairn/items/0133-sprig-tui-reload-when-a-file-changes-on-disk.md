---
id: 133
uid: 71b16b09-fb53-49ee-9659-dd9519b14bc4
title: 'sprig tui: reload when a file changes on disk'
type: feature
status: planned
milestone: v0.4
depends_on:
- 130
- 131
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: cli
---

## Problem

The point of a plain-text plan is that an editor, an agent and the TUI can all work on it at once. A TUI that holds a stale copy breaks that, and one that overwrites a newer file loses work.

## Proposal

Watch every file in the workspace, including grafted ones, and re-resolve on change while keeping the cursor on the same item by anchor, or by text and line. A write re-reads the file first. If the target line no longer holds the item the TUI thinks it does, it refuses the write and says why, as the spike (see its answer) decides. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] A test changes a file on disk while the core holds it and checks the new state appears and the cursor stays on its item.
- [ ] A test edits the target line externally between a keypress and its write, and checks the write is refused with nothing lost.
- [ ] Saving from Vim, which writes by rename, triggers a reload on macOS and Linux; checked by hand and noted here.
