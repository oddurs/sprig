---
id: 120
uid: 1eaf1d5b-13a6-4d93-8689-996fa5b4939f
title: 'sprig status: the whole folder on one screen'
type: feature
status: planned
milestone: v0.2
depends_on:
- 18
- 19
- 119
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: cli
---

## Problem

`tree` shows one file and `next` shows what's ready. Neither says how the whole folder is doing, and nothing fits in a shell prompt or a tmux status bar.

## Proposal

One line per file: a progress bar, done/total, how many items wait on something, how many are late. `--oneline` prints a single short line for a status bar, such as `sprig 23% · 21 ready · 11 waiting`. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] Output for `examples/bakery/` with `--today 2026-10-01` matches a committed snapshot.
- [ ] `--oneline` never prints more than 60 characters, and exits 1 when anything is late so a prompt can colour it.
- [ ] A file that fails to parse still gets a line, with its error count, instead of aborting the command.
