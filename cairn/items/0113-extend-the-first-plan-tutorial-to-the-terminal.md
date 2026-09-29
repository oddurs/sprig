---
id: 113
uid: ae3ed524-5793-42ec-ad1a-e1bc22e93d6e
title: Extend the first-plan tutorial to the terminal
type: docs
status: planned
milestone: launch
depends_on:
- 27
- 106
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: site
---

## Reader and question

A reader who finished the playground tutorial and wants the same plan checked by `sprig next` and `sprig check`.

## Change

Add the terminal steps to the tutorial once v0.1 exists: install, `sprig next`, `sprig check`, and a CI one-liner.

## Acceptance criteria

- [ ] Every command in the tutorial is run in CI against the tutorial's own plan.
