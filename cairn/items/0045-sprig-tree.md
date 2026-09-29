---
id: 45
uid: 178d65e6-1f89-4506-98d7-d3b4acb5824f
title: sprig tree
type: feature
status: planned
milestone: v0.2
depends_on:
- 18
- 22
- 119
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: cli
---

## Problem

There is no way to see a plan's shape and progress in the terminal, which the playground's Tree view shows at a glance.

## Proposal

Print a file as an indented tree with a status glyph, text, people and dates, and for each parent `done/total` plus the estimate left. `--depth N` and `--hide-done`.

## Acceptance criteria

- [ ] Output for `examples/bakery/bakery.sprig` matches a committed snapshot, with grafted branches shown under their graft line.
- [ ] Colour follows `--color auto|always|never` and `NO_COLOR`.
