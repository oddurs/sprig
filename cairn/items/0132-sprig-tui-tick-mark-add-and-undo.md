---
id: 132
uid: e8ad76db-cdd8-4417-b48a-f8f68fdec39e
title: 'sprig tui: tick, mark, add and undo'
type: feature
status: planned
milestone: v0.4
depends_on:
- 123
- 124
- 131
created: 2026-09-28
updated: 2026-09-28
priority: p0
effort: m
area: cli
---

## Problem

A TUI that can only read sends people back to the shell for every change.

## Proposal

`x` toggles done, `~` marks doing, `a` and `A` add a sibling or a child through a one-line prompt, `e` opens `$EDITOR` at the line and reloads afterwards, and `u` undoes this session's last write. Every write goes through the core's edit API and flashes `file:line  old → new` in the status line. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] After a scripted session of ticks, adds and undos on a copy of `examples/bakery/`, `git diff` shows exactly the lines the session changed, and undoing everything leaves no diff.
- [ ] Ticking an item that others wait on updates their blocked state in the same frame.
- [ ] Ticking a recurring item moves its due date, as `sprig tick` does.
