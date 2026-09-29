---
id: 134
uid: 42ee4ec0-91c5-4853-bf0e-2e06f662b0cc
title: 'sprig tui: next-up and agenda views, and filtering'
type: feature
status: planned
milestone: v0.4
depends_on:
- 23
- 127
- 131
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: cli
---

## Problem

The tree answers "where is everything". Day to day, the questions are "what now" and "what's due", which `next` and `agenda` already answer in the shell.

## Proposal

`1`, `2` and `3` switch between tree, next up and agenda. They use the same ranking and date walk as the CLI commands, so the two never disagree. `/` filters any view by text, as-you-type; `Esc` clears. `?` shows the help overlay. The plan is `design/cli-plan.html`.

## Acceptance criteria

- [ ] For `examples/bakery/`, the next-up view lists the same items in the same order as `sprig next`, checked by a test.
- [ ] Filtering the tree keeps the ancestors of every match visible, so matches stay in context.
- [ ] Snapshots of all three views are committed.
