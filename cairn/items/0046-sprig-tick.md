---
id: 46
uid: 5da6ef15-135b-43cd-860a-9a46e94cc1bd
title: sprig tick
type: feature
status: planned
milestone: v0.2
depends_on:
- 43
created: 2026-09-28
updated: 2026-09-28
priority: p1
effort: s
area: cli
---

## Problem

Scripts and people at a terminal need to tick items without opening an editor.

## Proposal

`sprig tick FILE:LINE`, `sprig tick ^id` or `sprig tick file^id` flips the mark between open and `x`. Recurring items roll their due date instead. Prints the changed line.

## Acceptance criteria

- [ ] Ticking a recurring item moves `due:` to the next occurrence and leaves the mark open, per the spec.
- [ ] Only the mark character (or the due date) changes, byte for byte.
- [ ] An ambiguous or unknown target exits 2 with the candidates listed.
